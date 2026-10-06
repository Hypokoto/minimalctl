use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
#[archive_attr(derive(bytecheck::CheckBytes))]
#[archive_attr(check_bytes(bound = "__C: rkyv::validation::ArchiveContext, <__C as rkyv::Fallible>::Error: std::error::Error"))]
#[archive(bound(
    serialize = "__S: rkyv::ser::Serializer + rkyv::ser::ScratchSpace",
    deserialize = "__D: rkyv::Fallible"
))]
pub enum SpatialNode {
    Pane {
        width: u16,
        height: u16,
        x: u16,
        y: u16,
        pane_id: u32,
    },
    HorizontalSplit {
        width: u16,
        height: u16,
        x: u16,
        y: u16,
        #[omit_bounds]
        #[archive_attr(omit_bounds)]
        children: Vec<SpatialNode>,
    },
    VerticalSplit {
        width: u16,
        height: u16,
        x: u16,
        y: u16,
        #[omit_bounds]
        #[archive_attr(omit_bounds)]
        children: Vec<SpatialNode>,
    },
}
