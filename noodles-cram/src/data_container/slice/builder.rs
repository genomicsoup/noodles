use std::{collections::HashMap, io};

use md5::{Digest, Md5};
use noodles_fasta as fasta;
use noodles_sam as sam;

use crate::{
    container::{
        block::{self, CompressionMethod},
        Block,
    },
    data_container::{
        compression_header::data_series_encoding_map::DataSeries, CompressionHeader,
        ReferenceSequenceContext,
    },
    record::Flags,
    writer, BitWriter, Record,
};

use super::{Header, Slice};

const CORE_DATA_BLOCK_CONTENT_ID: i32 = 0;
const MAX_RECORD_COUNT: usize = 10240;

#[derive(Debug, Default)]
pub struct Builder {
    records: Vec<Record>,
    reference_sequence_context: ReferenceSequenceContext,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddRecordError {
    SliceFull(Record),
}

impl Builder {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn reference_sequence_context(&self) -> ReferenceSequenceContext {
        self.reference_sequence_context
    }

    pub fn add_record(&mut self, record: Record) -> Result<&Record, AddRecordError> {
        if self.records.len() >= MAX_RECORD_COUNT {
            return Err(AddRecordError::SliceFull(record));
        }

        if self.is_empty() {
            self.reference_sequence_context = match (
                record.reference_sequence_id(),
                record.alignment_start(),
                record.alignment_end(),
            ) {
                (Some(id), Some(start), Some(end)) => {
                    ReferenceSequenceContext::some(id, start, end)
                }
                _ => ReferenceSequenceContext::None,
            };
        } else {
            self.reference_sequence_context.update(
                record.reference_sequence_id(),
                record.alignment_start(),
                record.alignment_end(),
            );
        };

        self.records.push(record);

        Ok(self.records.last().unwrap())
    }

    pub fn build(
        mut self,
        reference_sequence_repostitory: &fasta::repository::Repository,
        header: &sam::Header,
        compression_header: &CompressionHeader,
        record_counter: u64,
    ) -> io::Result<Slice> {
        let (core_data_block, external_blocks) = write_records(
            compression_header,
            self.reference_sequence_context,
            &mut self.records,
        )?;

        let mut block_content_ids = Vec::with_capacity(external_blocks.len() + 1);
        block_content_ids.push(core_data_block.content_id());

        for block in &external_blocks {
            block_content_ids.push(block.content_id());
        }

        let reference_md5 = match self.reference_sequence_context {
            ReferenceSequenceContext::Some(context) => {
                let reference_sequence_name = header
                    .reference_sequences()
                    .get_index(context.reference_sequence_id())
                    .map(|(_, rs)| rs.name())
                    .expect("invalid reference sequence ID");

                let reference_sequence = reference_sequence_repostitory
                    .get(reference_sequence_name)
                    .expect("missing reference sequence")
                    .expect("invalid reference sequence");

                let (start, end) = (context.alignment_start(), context.alignment_end());
                let sequence = &reference_sequence[start..=end];

                calculate_normalized_sequence_digest(sequence)
            }
            _ => [0; 16],
        };

        let header = Header::builder()
            .set_reference_sequence_context(self.reference_sequence_context)
            .set_record_count(self.records.len())
            .set_record_counter(record_counter)
            .set_block_count(block_content_ids.len())
            .set_block_content_ids(block_content_ids)
            .set_reference_md5(reference_md5)
            .build();

        Ok(Slice::new(header, core_data_block, external_blocks))
    }
}

fn write_records(
    compression_header: &CompressionHeader,
    reference_sequence_context: ReferenceSequenceContext,
    records: &mut [Record],
) -> io::Result<(Block, Vec<Block>)> {
    let mut core_data_writer = BitWriter::new(Vec::new());

    let mut external_data_writers = HashMap::new();

    for i in 0..DataSeries::LEN {
        let block_content_id = (i + 1) as i32;
        external_data_writers.insert(block_content_id, Vec::new());
    }

    for &block_content_id in compression_header.tag_encoding_map().keys() {
        external_data_writers.insert(block_content_id, Vec::new());
    }

    let mut record_writer = writer::record::Writer::new(
        compression_header,
        &mut core_data_writer,
        &mut external_data_writers,
        reference_sequence_context,
    );

    set_mates(records);

    for record in records {
        record_writer.write_record(record)?;
    }

    let core_data_block = core_data_writer.finish().and_then(|buf| {
        Block::builder()
            .set_content_type(block::ContentType::CoreData)
            .set_content_id(CORE_DATA_BLOCK_CONTENT_ID)
            .compress_and_set_data(buf, CompressionMethod::Gzip)
            .map(|builder| builder.build())
    })?;

    let external_blocks: Vec<_> = external_data_writers
        .into_iter()
        .filter(|(_, buf)| !buf.is_empty())
        .map(|(block_content_id, buf)| {
            Block::builder()
                .set_content_type(block::ContentType::ExternalData)
                .set_content_id(block_content_id)
                .compress_and_set_data(buf, CompressionMethod::Gzip)
                .map(|builder| builder.build())
        })
        .collect::<Result<_, _>>()?;

    Ok((core_data_block, external_blocks))
}

fn set_mates(records: &mut [Record]) {
    assert!(!records.is_empty());

    let mut indices = HashMap::new();
    let mut i = records.len() - 1;

    loop {
        let record = &mut records[i];
        let flags = record.flags();

        if flags.is_segmented() && !flags.is_secondary() {
            let read_name = record.read_name().cloned();

            if let Some(j) = indices.insert(read_name, i) {
                let mid = i + 1;
                let (left, right) = records.split_at_mut(mid);

                let record = &mut left[i];
                let mate = &mut right[j - mid];

                set_downstream_mate(i, record, j, mate);
            } else {
                set_detached(record);
            }
        } else {
            set_detached(record);
        }

        if i == 0 {
            break;
        }

        i -= 1;
    }
}

fn set_downstream_mate(i: usize, record: &mut Record, j: usize, mate: &mut Record) {
    record.distance_to_next_fragment = Some(j - i - 1);
    record.cram_bit_flags.insert(Flags::HAS_MATE_DOWNSTREAM);
    mate.cram_bit_flags.remove(Flags::DETACHED);
}

fn set_detached(record: &mut Record) {
    record.cram_bit_flags.insert(Flags::DETACHED);
}

// _Sequence Alignment/Map Format Specification_ (2021-06-03) § 1.3.2 "Reference MD5 calculation"
pub(super) fn calculate_normalized_sequence_digest(sequence: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();

    for &b in sequence {
        // "All characters outside of the inclusive range 33 ('!') to 126 ('~') are stripped out."
        if b.is_ascii_graphic() {
            // "All lowercase characters are converted to uppercase."
            hasher.update(&[b.to_ascii_uppercase()]);
        }
    }

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_normalized_sequence_digest() {
        assert_eq!(
            calculate_normalized_sequence_digest(b"ACGT"),
            [
                0xf1, 0xf8, 0xf4, 0xbf, 0x41, 0x3b, 0x16, 0xad, 0x13, 0x57, 0x22, 0xaa, 0x45, 0x91,
                0x04, 0x3e
            ]
        );

        assert_eq!(
            calculate_normalized_sequence_digest(b"ACgt"),
            [
                0xf1, 0xf8, 0xf4, 0xbf, 0x41, 0x3b, 0x16, 0xad, 0x13, 0x57, 0x22, 0xaa, 0x45, 0x91,
                0x04, 0x3e
            ]
        );

        // _Sequence Alignment/Map Format Specification_ (2021-06-03) § 1.3.2 "Reference MD5
        // calculation"
        assert_eq!(
            calculate_normalized_sequence_digest(b"ACGTACGTACGTACGTACGTACGT...12345!!!"),
            [
                0xdf, 0xab, 0xdb, 0xb3, 0x6e, 0x23, 0x9a, 0x6d, 0xa8, 0x89, 0x57, 0x84, 0x1f, 0x32,
                0xb8, 0xe4
            ]
        );
    }
}
