use noodles_core::Position;
use noodles_sam::{
    self as sam,
    record::{quality_scores::Score, sequence::Base},
};

use super::{Feature, Features, Flags, NextMateFlags, Record};

/// A CRAM record builder.
pub struct Builder {
    id: u64,
    bam_flags: sam::record::Flags,
    flags: Flags,
    reference_sequence_id: Option<usize>,
    read_length: usize,
    alignment_start: Option<Position>,
    read_group_id: Option<usize>,
    read_name: Option<sam::record::ReadName>,
    next_mate_flags: NextMateFlags,
    next_fragment_reference_sequence_id: Option<usize>,
    next_mate_alignment_start: Option<Position>,
    template_size: i32,
    distance_to_next_fragment: Option<usize>,
    tags: sam::record::Data,
    bases: sam::record::Sequence,
    features: Features,
    mapping_quality: Option<sam::record::MappingQuality>,
    quality_scores: sam::record::QualityScores,
}

impl Builder {
    /// Sets the CRAM record ID.
    pub fn set_id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    /// Sets the BAM flags.
    pub fn set_bam_flags(mut self, bam_flags: sam::record::Flags) -> Self {
        self.bam_flags = bam_flags;
        self
    }

    /// Sets the CRAM flags.
    pub fn set_flags(mut self, flags: Flags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the reference sequence ID.
    pub fn set_reference_sequence_id(mut self, reference_sequence_id: usize) -> Self {
        self.reference_sequence_id = Some(reference_sequence_id);
        self
    }

    /// Sets the read length.
    pub fn set_read_length(mut self, read_length: usize) -> Self {
        self.read_length = read_length;
        self
    }

    /// Sets the alignment start position.
    pub fn set_alignment_start(mut self, alignment_start: Position) -> Self {
        self.alignment_start = Some(alignment_start);
        self
    }

    /// Sets the read group ID.
    pub fn set_read_group_id(mut self, read_group_id: usize) -> Self {
        self.read_group_id = Some(read_group_id);
        self
    }

    /// Sets the read name.
    pub fn set_read_name(mut self, read_name: sam::record::ReadName) -> Self {
        self.read_name = Some(read_name);
        self
    }

    /// Sets the next mate flags.
    pub fn set_next_mate_flags(mut self, next_mate_flags: NextMateFlags) -> Self {
        self.next_mate_flags = next_mate_flags;
        self
    }

    /// Sets the reference sequence ID of the next fragment.
    pub fn set_next_fragment_reference_sequence_id(
        mut self,
        next_fragment_reference_sequence_id: usize,
    ) -> Self {
        self.next_fragment_reference_sequence_id = Some(next_fragment_reference_sequence_id);
        self
    }

    /// Sets the alignment start position of the next mate.
    pub fn set_next_mate_alignment_start(mut self, next_mate_alignment_start: Position) -> Self {
        self.next_mate_alignment_start = Some(next_mate_alignment_start);
        self
    }

    /// Sets the template size.
    pub fn set_template_size(mut self, template_size: i32) -> Self {
        self.template_size = template_size;
        self
    }

    /// Sets the distance to the next fragment.
    pub fn set_distance_to_next_fragment(mut self, distance_to_next_fragment: usize) -> Self {
        self.distance_to_next_fragment = Some(distance_to_next_fragment);
        self
    }

    /// Sets the tag dictionary.
    pub fn set_tags(mut self, tags: sam::record::Data) -> Self {
        self.tags = tags;
        self
    }

    /// Adds a tag to the tag dictionary.
    pub fn add_tag(mut self, tag: sam::record::data::Field) -> Self {
        self.tags.insert(tag);
        self
    }

    /// Sets the read bases.
    pub fn set_bases(mut self, bases: sam::record::Sequence) -> Self {
        self.bases = bases;
        self
    }

    /// Adds a base to the read bases.
    pub fn add_base(mut self, base: Base) -> Self {
        self.bases.push(base);
        self
    }

    /// Sets the read features.
    pub fn set_features(mut self, features: Features) -> Self {
        self.features = features;
        self
    }

    /// Adds a read feature.
    pub fn add_feature(mut self, feature: Feature) -> Self {
        self.features.push(feature);
        self
    }

    /// Sets the mapping quality.
    pub fn set_mapping_quality(mut self, mapping_quality: sam::record::MappingQuality) -> Self {
        self.mapping_quality = Some(mapping_quality);
        self
    }

    /// Sets the per-base quality scores.
    pub fn set_quality_scores(mut self, quality_scores: sam::record::QualityScores) -> Self {
        self.quality_scores = quality_scores;
        self
    }

    /// Adds a quality score.
    pub fn add_quality_score(mut self, quality_score: Score) -> Self {
        self.quality_scores.push(quality_score);
        self
    }

    /// Builds a CRAM record.
    pub fn build(self) -> Record {
        Record {
            id: self.id,
            bam_bit_flags: self.bam_flags,
            cram_bit_flags: self.flags,
            reference_sequence_id: self.reference_sequence_id,
            read_length: self.read_length,
            alignment_start: self.alignment_start,
            read_group: self.read_group_id,
            read_name: self.read_name,
            next_mate_bit_flags: self.next_mate_flags,
            next_fragment_reference_sequence_id: self.next_fragment_reference_sequence_id,
            next_mate_alignment_start: self.next_mate_alignment_start,
            template_size: self.template_size,
            distance_to_next_fragment: self.distance_to_next_fragment,
            tags: self.tags,
            bases: self.bases,
            features: self.features,
            mapping_quality: self.mapping_quality,
            quality_scores: self.quality_scores,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            id: 0,
            bam_flags: sam::record::Flags::UNMAPPED,
            flags: Flags::default(),
            reference_sequence_id: None,
            read_length: 0,
            alignment_start: None,
            read_group_id: None,
            read_name: None,
            next_mate_flags: NextMateFlags::default(),
            next_fragment_reference_sequence_id: None,
            next_mate_alignment_start: None,
            template_size: 0,
            distance_to_next_fragment: None,
            tags: sam::record::Data::default(),
            bases: sam::record::Sequence::default(),
            features: Features::default(),
            mapping_quality: None,
            quality_scores: sam::record::QualityScores::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let builder = Builder::default();

        assert_eq!(builder.id, 0);
        assert_eq!(builder.bam_flags, sam::record::Flags::UNMAPPED);
        assert_eq!(builder.flags, Flags::default());
        assert!(builder.reference_sequence_id.is_none());
        assert_eq!(builder.read_length, 0);
        assert!(builder.alignment_start.is_none());
        assert!(builder.read_group_id.is_none());
        assert!(builder.read_name.is_none());
        assert_eq!(builder.next_mate_flags, NextMateFlags::default());
        assert!(builder.next_fragment_reference_sequence_id.is_none());
        assert!(builder.next_mate_alignment_start.is_none());
        assert_eq!(builder.template_size, 0);
        assert!(builder.distance_to_next_fragment.is_none());
        assert!(builder.tags.is_empty());
        assert!(builder.bases.is_empty());
        assert!(builder.features.is_empty());
        assert!(builder.mapping_quality.is_none());
        assert!(builder.quality_scores.is_empty());
    }
}
