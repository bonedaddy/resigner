pub mod rpc_utils;

use bytemuck::cast_slice;

use bytemuck::{from_bytes, Pod};
pub fn load<T: Pod>(data: &[u8]) -> anyhow::Result<&T> {
    let _size = std::mem::size_of::<T>();
    Ok(from_bytes(cast_slice::<u8, u8>(&data[0..])))
}
