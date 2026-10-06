use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
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
        children: Vec<SpatialNode>,
    },
    VerticalSplit {
        width: u16,
        height: u16,
        x: u16,
        y: u16,
        #[omit_bounds]
        children: Vec<SpatialNode>,
    },
}

impl<C: ?Sized + rkyv::Fallible> bytecheck::CheckBytes<C> for ArchivedSpatialNode
where
    <C as rkyv::Fallible>::Error: std::error::Error,
{
    type Error = C::Error;
    unsafe fn check_bytes<'a>(
        value: *const Self,
        _context: &mut C,
    ) -> Result<&'a Self, C::Error> {
        Ok(&*value)
    }
}
