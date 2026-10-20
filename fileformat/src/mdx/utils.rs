use std::{
    io::{self, Read},
    mem::{MaybeUninit, size_of},
    sync::Arc,
};

pub(super) fn read_value<T>(reader: &mut impl Read) -> io::Result<T> {
    let mut value = MaybeUninit::<T>::uninit();
    let bytes =
        unsafe { std::slice::from_raw_parts_mut(value.as_mut_ptr().cast(), size_of::<T>()) };
    reader.read_exact(bytes)?;
    Ok(unsafe { value.assume_init() })
}

pub(super) fn read_values<T>(reader: &mut impl Read, count: usize) -> io::Result<Vec<T>> {
    let size = count * size_of::<T>();
    if !size.is_multiple_of(size_of::<T>()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid global sequence size",
        ));
    }

    let mut values: Vec<T> = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "list allocation is too large"))?;
    let byte_len = count
        .checked_mul(size_of::<T>())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "list size overflow"))?;
    let bytes = unsafe { std::slice::from_raw_parts_mut(values.as_mut_ptr().cast(), byte_len) };
    reader.read_exact(bytes)?;
    unsafe { values.set_len(count) };
    Ok(values)
}

pub(super) fn read_list<T>(reader: &mut impl Read) -> io::Result<Vec<T>> {
    let count = read_value::<u32>(reader)? as usize;
    read_values(reader, count)
}

pub(super) fn fixed_string(reader: &mut impl Read, len: usize) -> io::Result<Arc<str>> {
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes)?;
    Ok(fixed_string_bytes(&bytes))
}

pub(super) fn fixed_string_bytes(bytes: &[u8]) -> Arc<str> {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    Arc::from(String::from_utf8_lossy(&bytes[..end]).as_ref())
}

pub fn animation_ratio(anim: &str) -> Option<f32> {
    let (_, ratio) = anim.rsplit_once('@')?;
    let ratio = ratio.parse::<f32>().ok()?;

    ratio.is_finite().then(|| ratio.clamp(0., 1.))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn fixed_string_stops_at_null() {
        assert_eq!(
            &*fixed_string(&mut Cursor::new(b"Stand\0ignored"), 13).unwrap(),
            "Stand"
        );
        assert_eq!(
            &*fixed_string(&mut Cursor::new(b"Walk"), 4).unwrap(),
            "Walk"
        );
    }

    #[test]
    fn reads_value_bytes() {
        assert_eq!(
            read_value::<u32>(&mut Cursor::new(42_u32.to_le_bytes())).unwrap(),
            42
        );
    }

    #[test]
    fn reads_counted_list() {
        let mut bytes = 2_u32.to_le_bytes().to_vec();
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(4_u16.to_le_bytes());

        assert_eq!(read_list::<u16>(&mut Cursor::new(bytes)).unwrap(), [3, 4]);
    }
}
