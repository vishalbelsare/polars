use crate::trusted_length::TrustMyLength;
use arrow::array::{Array, ArrayData, BooleanArray};
use arrow::buffer::Buffer;
use arrow::datatypes::DataType;

fn prepare_buffers(
    l: &Buffer,
    offset_l: usize,
    r: &Buffer,
    offset_r: usize,
    len_in_bits: usize,
) -> (BooleanArray, BooleanArray) {
    let l = BooleanArray::from(
        ArrayData::builder(DataType::Boolean)
            .add_buffer(l.clone())
            .offset(offset_l)
            .len(len_in_bits)
            .build(),
    );
    let r = BooleanArray::from(
        ArrayData::builder(DataType::Boolean)
            .add_buffer(r.clone())
            .offset(offset_r)
            .len(len_in_bits)
            .build(),
    );
    (l, r)
}

// buffer ops from arrow is private. So we hack around this by creating boolean arrays.
// should do the same as arrows buffer_bin_and
pub fn buffer_and(
    l: &Buffer,
    offset_l: usize,
    r: &Buffer,
    offset_r: usize,
    len_in_bits: usize,
) -> Buffer {
    let (l, r) = prepare_buffers(l, offset_l, r, offset_r, len_in_bits);
    (arrow::compute::and(&l, &r).unwrap()).data().buffers()[0].clone()
}

pub fn buffer_or(
    l: &Buffer,
    offset_l: usize,
    r: &Buffer,
    offset_r: usize,
    len_in_bits: usize,
) -> Buffer {
    let (l, r) = prepare_buffers(l, offset_l, r, offset_r, len_in_bits);
    (arrow::compute::or(&l, &r).unwrap()).data().buffers()[0].clone()
}

/// Combine the validity by doing a bitand operation.
pub fn combine_null_buffers(
    opt_l: Option<&Buffer>,
    offset_l: usize,
    opt_r: Option<&Buffer>,
    offset_r: usize,
    len_in_bits: usize,
) -> Option<Buffer> {
    match (opt_l, opt_r) {
        (Some(l), Some(r)) => Some(buffer_and(l, offset_l, r, offset_r, len_in_bits)),
        (Some(l), None) => Some(l.clone()),
        (None, Some(r)) => Some(r.clone()),
        (None, None) => None,
    }
}

pub trait CustomIterTools: Iterator {
    fn fold_first_<F>(mut self, f: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        let first = self.next()?;
        Some(self.fold(first, f))
    }

    fn trust_my_length(self, length: usize) -> TrustMyLength<Self, Self::Item>
    where
        Self: Sized,
    {
        TrustMyLength::new(self, length)
    }
}

impl<T: ?Sized> CustomIterTools for T where T: Iterator {}
