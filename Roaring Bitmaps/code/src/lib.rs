use std::cmp::{max, max_by_key, min_by_key};

#[derive(Debug)]
struct RoaringBitmap {
    data: Vec<Container>,
}

impl RoaringBitmap {
    fn new() -> RoaringBitmap {
        RoaringBitmap {
            data: Vec::new(),
        }
    }

    fn add(&mut self, value: u32) {
        let most_significant_bits = (value >> 16) as u16;
        let least_significant_bits = value as u16;
        match self.data.binary_search_by_key(&most_significant_bits, |container| container.most_significant_bits) {
            Ok(index) => {
                let mut container = std::mem::replace(&mut self.data[index].container, Default::default());
                container = container.add(least_significant_bits);
                self.data[index].container = container;
            }
            Err(index) => {
                let mut container: ContainerType = Default::default();
                container = container.add(least_significant_bits);
                self.data.insert(index, Container { most_significant_bits, container });
            }
        };
    }

    fn contains(&self, value: u32) -> bool {
        let most_significant_bits = (value >> 16) as u16;
        let least_significant_bits = value as u16;
        match self.data.binary_search_by_key(&most_significant_bits, |container| container.most_significant_bits) {
            Ok(index) => self.data[index].container.contains(least_significant_bits),
            Err(_) => false
        }
    }

    fn remove(&mut self, value: u32) {
        let most_significant_bits = (value >> 16) as u16;
        if let Ok(index) = self.data.binary_search_by_key(&most_significant_bits, |container| container.most_significant_bits) {
            let least_significant_bits = value as u16;
            let mut container = std::mem::replace(&mut self.data[index].container, Default::default());
            container = container.remove(least_significant_bits);
            self.data[index].container = container;
        }
    }

    fn union(&self, rhs: &RoaringBitmap) -> Self {
        let mut res = RoaringBitmap::new();
        let mut lhs_idx = 0;
        let mut rhs_idx = 0;
        while lhs_idx < self.data.len() && rhs_idx < rhs.data.len() {
            let lhs_container = &self.data[lhs_idx];
            let rhs_container = &rhs.data[rhs_idx];
            if lhs_container.most_significant_bits == rhs_container.most_significant_bits {
                res.data.push(Container {
                    most_significant_bits: lhs_container.most_significant_bits,
                    container: lhs_container.container.union(&rhs_container.container),
                });
                lhs_idx += 1;
                rhs_idx += 1;
            } else if lhs_container.most_significant_bits < rhs_container.most_significant_bits {
                res.data.push(lhs_container.clone());
                lhs_idx += 1;
            } else {
                res.data.push(rhs_container.clone());
                rhs_idx += 1;
            }
        }
        while lhs_idx < self.data.len() {
            res.data.push(self.data[lhs_idx].clone());
            lhs_idx += 1;
        }
        while rhs_idx < rhs.data.len() {
            res.data.push(rhs.data[rhs_idx].clone());
            rhs_idx += 1;
        }
        res
    }

    fn intersection(&self, rhs: &RoaringBitmap) -> Self {
        let mut res = RoaringBitmap::new();
        let mut lhs_idx = 0;
        let mut rhs_idx = 0;
        while lhs_idx < self.data.len() && rhs_idx < rhs.data.len() {
            let lhs_container = &self.data[lhs_idx];
            let rhs_container = &rhs.data[rhs_idx];
            if lhs_container.most_significant_bits == rhs_container.most_significant_bits {
                let container = lhs_container.container.intersection(&rhs_container.container);
                if !container.is_empty() {
                    res.data.push(Container {
                        most_significant_bits: lhs_container.most_significant_bits,
                        container,
                    });
                }
                lhs_idx += 1;
                rhs_idx += 1;
            } else if lhs_container.most_significant_bits < rhs_container.most_significant_bits {
                lhs_idx += 1;
            } else {
                rhs_idx += 1;
            }
        }
        res
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Debug)]
struct Container {
    most_significant_bits: u16,
    container: ContainerType,
}

impl Clone for Container {
    fn clone(&self) -> Self {
        Container {
            most_significant_bits: self.most_significant_bits,
            container: self.container.clone(),
        }
    }
}

#[derive(Debug)]
enum ContainerType {
    ContainerTypeArray(ArrayContainer),
    ContainerTypeBitmap(BitmapContainer),
}

impl Default for ContainerType {
    fn default() -> Self { ContainerType::ContainerTypeArray(ArrayContainer::new(0)) }
}

impl ContainerType {
    fn add(self, value: u16) -> Self {
        match self {
            Self::ContainerTypeArray(mut array_container) => {
                // This check ensures that duplicate values do not cause unnecessary
                // container conversion, because duplicate values will not increase the
                // container size beyond 4096.
                if array_container.array.len() == 4096 && !array_container.contains(&value) {
                    let mut bitmap_container = BitmapContainer::from(array_container);
                    bitmap_container.add(value);
                    Self::ContainerTypeBitmap(bitmap_container)
                } else {
                    array_container.add(value);
                    Self::ContainerTypeArray(array_container)
                }
            }
            Self::ContainerTypeBitmap(mut bitmap_container) => {
                bitmap_container.add(value);
                Self::ContainerTypeBitmap(bitmap_container)
            }
        }
    }

    fn contains(&self, value: u16) -> bool {
        match self {
            Self::ContainerTypeArray(array_container) => array_container.contains(&value),
            Self::ContainerTypeBitmap(bitmap_container) => bitmap_container.contains(&value),
        }
    }

    fn remove(self, value: u16) -> Self {
        match self {
            Self::ContainerTypeArray(mut array_container) => {
                array_container.remove(&value);
                Self::ContainerTypeArray(array_container)
            }
            Self::ContainerTypeBitmap(mut bitmap_container) => {
                bitmap_container.remove(&value);
                if bitmap_container.cardinality == 4096 {
                    let array_container = ArrayContainer::from(bitmap_container);
                    Self::ContainerTypeArray(array_container)
                } else {
                    Self::ContainerTypeBitmap(bitmap_container)
                }
            }
        }
    }

    fn union(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::ContainerTypeArray(lhs), Self::ContainerTypeArray(rhs)) => {
                Self::union_array_array(lhs, rhs)
            }
            (Self::ContainerTypeBitmap(lhs), Self::ContainerTypeBitmap(rhs)) => {
                Self::union_bitmap_bitmap(lhs, rhs)
            }
            (Self::ContainerTypeArray(lhs), Self::ContainerTypeBitmap(rhs)) => {
                Self::union_array_bitmap(lhs, rhs)
            }
            (Self::ContainerTypeBitmap(lhs), Self::ContainerTypeArray(rhs)) => {
                Self::union_array_bitmap(rhs, lhs)
            }
        }
    }

    fn intersection(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::ContainerTypeArray(lhs), Self::ContainerTypeArray(rhs)) => {
                Self::intersection_array_array(lhs, rhs)
            }
            (Self::ContainerTypeBitmap(lhs), Self::ContainerTypeBitmap(rhs)) => {
                Self::intersection_bitmap_bitmap(lhs, rhs)
            }
            (Self::ContainerTypeArray(lhs), Self::ContainerTypeBitmap(rhs)) => {
                Self::intersection_array_bitmap(lhs, rhs)
            }
            (Self::ContainerTypeBitmap(lhs), Self::ContainerTypeArray(rhs)) => {
                Self::intersection_array_bitmap(rhs, lhs)
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::ContainerTypeArray(array_container) => array_container.array.is_empty(),
            Self::ContainerTypeBitmap(bitmap_container) => bitmap_container.cardinality == 0,
        }
    }
}

impl Clone for ContainerType {
    fn clone(&self) -> Self {
        match self {
            Self::ContainerTypeArray(array_container) => Self::ContainerTypeArray(array_container.clone()),
            Self::ContainerTypeBitmap(bitmap_container) => Self::ContainerTypeBitmap(bitmap_container.clone()),
        }
    }
}

#[derive(Debug)]
struct ArrayContainer {
    most_significant_bits: u16,
    array: Vec<u16>,
}

impl ArrayContainer {
    fn new(most_significant_bits: u16) -> Self {
        ArrayContainer {
            most_significant_bits,
            array: Vec::new(),
        }
    }
    fn add(&mut self, value: u16) {
        self.array.binary_search(&value).err().map(|index| self.array.insert(index, value));
    }

    fn contains(&self, value: &u16) -> bool {
        self.array.binary_search(value).is_ok()
    }

    fn remove(&mut self, value: &u16) {
        self.array.binary_search(value).ok().map(|index| self.array.remove(index));
    }
}

impl From<BitmapContainer> for ArrayContainer {
    fn from(bitmap_container: BitmapContainer) -> Self {
        let mut array = Vec::with_capacity(bitmap_container.cardinality as usize);
        for (index, bitmap) in bitmap_container.bitmap.iter().enumerate() {
            if *bitmap == 0 {
                continue;
            }

            let indices = Self::extract_set_bit_indices(bitmap.clone());
            for offset in indices {
                array.push((index * 64 + offset) as u16);
            }
        }
        ArrayContainer {
            most_significant_bits: bitmap_container.most_significant_bits,
            array,
        }
    }
}

impl ArrayContainer {
    fn extract_set_bit_indices(mut bitmap: u64) -> Vec<usize> {
        let mut indices = Vec::with_capacity(bitmap.count_ones() as usize);
        while bitmap != 0 {
            let lsb_value = bitmap & (!bitmap + 1);
            let lsb_bit_index = lsb_value.trailing_zeros() as usize;
            indices.push(lsb_bit_index);
            bitmap &= bitmap - 1;
        }
        indices
    }
}

impl Clone for ArrayContainer {
    fn clone(&self) -> Self {
        ArrayContainer {
            most_significant_bits: self.most_significant_bits,
            array: self.array.clone(),
        }
    }
}

#[derive(Debug)]
struct BitmapContainer {
    most_significant_bits: u16,
    cardinality: usize,
    bitmap: Vec<u64>,
}

impl BitmapContainer {
    fn add(&mut self, value: u16) {
        let (index, offset) = Self::get_index_and_offset(&value);
        self.bitmap.resize(index + 1, 0);
        let before_bitwise_op = self.bitmap[index];
        self.bitmap[index] |= 1 << offset;
        let after_bitwise_op = self.bitmap[index];
        self.cardinality += (before_bitwise_op != after_bitwise_op) as usize;
    }

    fn contains(&self, value: &u16) -> bool {
        let (index, offset) = Self::get_index_and_offset(value);
        if index >= self.bitmap.len() {
            return false;
        }
        self.bitmap[index] & (1 << offset) != 0
    }

    fn remove(&mut self, value: &u16) {
        let (index, offset) = Self::get_index_and_offset(value);
        let before_bitwise_op = self.bitmap[index];
        self.bitmap[index] &= !(1 << offset);
        let after_bitwise_op = self.bitmap[index];
        self.cardinality -= (before_bitwise_op != after_bitwise_op) as usize;
    }

    fn get_index_and_offset(value: &u16) -> (usize, usize) {
        (*value as usize / 64, *value as usize % 64)
    }
}

impl From<ArrayContainer> for BitmapContainer {
    fn from(array_container: ArrayContainer) -> Self {
        let mut bitmap = vec![0; 1 + array_container.array.last().map_or(0, |&x| x as usize / 64)];
        for &value in &array_container.array {
            let (index, offset) = BitmapContainer::get_index_and_offset(&value);
            bitmap[index] |= 1 << offset;
        }
        BitmapContainer {
            most_significant_bits: array_container.most_significant_bits,
            bitmap,
            cardinality: array_container.array.len(),
        }
    }
}

impl Clone for BitmapContainer {
    fn clone(&self) -> Self {
        BitmapContainer {
            most_significant_bits: self.most_significant_bits,
            cardinality: self.cardinality,
            bitmap: self.bitmap.clone(),
        }
    }
}

impl ContainerType {
    fn union_array_array(lhs: &ArrayContainer, rhs: &ArrayContainer) -> Self {
        let sum = lhs.array.len() + rhs.array.len();
        if sum > 4096 {
            let mut res = BitmapContainer {
                most_significant_bits: lhs.most_significant_bits,
                cardinality: 0,
                bitmap: Vec::with_capacity(1 + sum / 64),
            };
            let mut lhs_idx = 0;
            let mut rhs_idx = 0;
            while lhs_idx < lhs.array.len() && rhs_idx < rhs.array.len() {
                let lhs_value = lhs.array[lhs_idx];
                let rhs_value = rhs.array[rhs_idx];
                if lhs_value == rhs_value {
                    res.add(lhs_value);
                    lhs_idx += 1;
                    rhs_idx += 1;
                } else if lhs_value < rhs_value {
                    res.add(lhs_value);
                    lhs_idx += 1;
                } else {
                    res.add(rhs_value);
                    rhs_idx += 1;
                }
            }
            while lhs_idx < lhs.array.len() {
                res.add(lhs.array[lhs_idx]);
                lhs_idx += 1;
            }
            while rhs_idx < rhs.array.len() {
                res.add(rhs.array[rhs_idx]);
                rhs_idx += 1;
            }
            if res.cardinality > 4096 {
                Self::ContainerTypeBitmap(res)
            } else {
                Self::ContainerTypeArray(ArrayContainer::from(res))
            }
        } else {
            let mut res = ArrayContainer {
                most_significant_bits: lhs.most_significant_bits,
                array: Vec::with_capacity(sum),
            };
            let mut lhs_idx = 0;
            let mut rhs_idx = 0;
            while lhs_idx < lhs.array.len() && rhs_idx < rhs.array.len() {
                let lhs_value = lhs.array[lhs_idx];
                let rhs_value = rhs.array[rhs_idx];
                if lhs_value == rhs_value {
                    res.array.push(lhs_value);
                    lhs_idx += 1;
                    rhs_idx += 1;
                } else if lhs_value < rhs_value {
                    res.array.push(lhs_value);
                    lhs_idx += 1;
                } else {
                    res.array.push(rhs_value);
                    rhs_idx += 1;
                }
            }
            while lhs_idx < lhs.array.len() {
                res.array.push(lhs.array[lhs_idx]);
                lhs_idx += 1;
            }
            while rhs_idx < rhs.array.len() {
                res.array.push(rhs.array[rhs_idx]);
                rhs_idx += 1;
            }
            Self::ContainerTypeArray(res)
        }
    }

    fn union_bitmap_bitmap(lhs: &BitmapContainer, rhs: &BitmapContainer) -> Self {
        const SELECT: bool = true;
        if SELECT {
            Self::approach_paper(lhs, rhs)
        } else {
            Self::approach_custom(lhs, rhs)
        }
    }

    /*
    This is an Alternate approach wherein we take a clone of the larger of lhs and rhs then iterate over the smaller of lhs and rhs
    and add the values to the clone (don't use the add function but do bitwise OR). This approach is more efficient because it avoids resizing the bitmap vector.
    Also to get the correct value of cardinality, we just need to compare the change in set bits for each u64 value,
    before and after the bitwise OR operations.
     */
    fn approach_custom(lhs: &BitmapContainer, rhs: &BitmapContainer) -> ContainerType {
        let mut res = max_by_key(lhs, rhs, |&container| container.bitmap.len()).clone();
        let smaller = min_by_key(lhs, rhs, |&container| container.bitmap.len()).clone();
        for (index, rhs_bitmap) in smaller.bitmap.iter().enumerate() {
            let original_set_bits_count = res.bitmap[index].count_ones();
            res.bitmap[index] |= rhs_bitmap;
            let modified_set_bits_count = res.bitmap[index].count_ones();
            res.cardinality += (modified_set_bits_count - original_set_bits_count) as usize;
        }
        Self::ContainerTypeBitmap(res)
    }

    fn approach_paper(lhs: &BitmapContainer, rhs: &BitmapContainer) -> ContainerType {
        let mut res = BitmapContainer {
            most_significant_bits: lhs.most_significant_bits,
            cardinality: 0,
            bitmap: Vec::with_capacity(max(lhs.bitmap.len(), rhs.bitmap.len())),
        };
        let mut lhs_idx = 0;
        let mut rhs_idx = 0;
        while lhs_idx < lhs.bitmap.len() && rhs_idx < rhs.bitmap.len() {
            let lhs_bitmap = lhs.bitmap[lhs_idx];
            let rhs_bitmap = rhs.bitmap[rhs_idx];
            let union_bitmap = lhs_bitmap | rhs_bitmap;
            res.bitmap.push(union_bitmap);
            res.cardinality += union_bitmap.count_ones() as usize;
            lhs_idx += 1;
            rhs_idx += 1;
        }
        while lhs_idx < lhs.bitmap.len() {
            let lhs_bitmap = lhs.bitmap[lhs_idx];
            res.bitmap.push(lhs_bitmap);
            res.cardinality += lhs_bitmap.count_ones() as usize;
            lhs_idx += 1;
        }
        while rhs_idx < rhs.bitmap.len() {
            let rhs_bitmap = rhs.bitmap[rhs_idx];
            res.bitmap.push(rhs_bitmap);
            res.cardinality += rhs_bitmap.count_ones() as usize;
            rhs_idx += 1;
        }
        Self::ContainerTypeBitmap(res)
    }

    fn union_array_bitmap(lhs: &ArrayContainer, rhs: &BitmapContainer) -> Self {
        let mut res = rhs.clone();
        for value in lhs.array.iter() {
            res.add(*value);
        }
        Self::ContainerTypeBitmap(res)
    }

    // TODO: Use Galloping intersections
    fn intersection_array_array(lhs: &ArrayContainer, rhs: &ArrayContainer) -> Self {
        let mut res = ArrayContainer {
            most_significant_bits: lhs.most_significant_bits,
            array: Vec::new(),
        };
        let (smaller_array, larger_array) = if lhs.array.len() > rhs.array.len() {
            (rhs, lhs)
        } else {
            (lhs, rhs)
        };
        for value in smaller_array.array.iter() {
            if larger_array.contains(value) {
                res.array.push(*value);
            }
        }
        Self::ContainerTypeArray(res)
    }

    fn intersection_bitmap_bitmap(lhs: &BitmapContainer, rhs: &BitmapContainer) -> Self {
        let mut cardinality = 0;
        let mut idx = 0;
        while idx < lhs.bitmap.len() && idx < rhs.bitmap.len() {
            let lhs_bitmap = lhs.bitmap[idx];
            let rhs_bitmap = rhs.bitmap[idx];
            let union_bitmap = lhs_bitmap & rhs_bitmap;
            cardinality += union_bitmap.count_ones() as usize;
            idx += 1;
        }

        if cardinality > 4096 {
            let mut res = BitmapContainer {
                most_significant_bits: lhs.most_significant_bits,
                cardinality,
                bitmap: Vec::with_capacity(max(lhs.bitmap.len(), rhs.bitmap.len())),
            };
            let mut idx = 0;
            while idx < lhs.bitmap.len() && idx < rhs.bitmap.len() {
                let lhs_bitmap = lhs.bitmap[idx];
                let rhs_bitmap = rhs.bitmap[idx];
                let intersection_bitmap = lhs_bitmap & rhs_bitmap;
                res.bitmap.push(intersection_bitmap);
                idx += 1;
            }
            Self::ContainerTypeBitmap(res)
        } else {
            let mut res = ArrayContainer {
                most_significant_bits: lhs.most_significant_bits,
                array: Vec::with_capacity(cardinality),
            };
            let mut idx = 0;
            while idx < lhs.bitmap.len() && idx < rhs.bitmap.len() {
                let lhs_bitmap = lhs.bitmap[idx];
                let rhs_bitmap = rhs.bitmap[idx];
                let intersection_bitmap = lhs_bitmap & rhs_bitmap;
                let indices = ArrayContainer::extract_set_bit_indices(intersection_bitmap);
                for offset in indices {
                    res.array.push((idx * 64 + offset) as u16);
                }
                idx += 1;
            }
            Self::ContainerTypeArray(res)
        }
    }

    fn intersection_array_bitmap(lhs: &ArrayContainer, rhs: &BitmapContainer) -> Self {
        let mut res = ArrayContainer {
            most_significant_bits: lhs.most_significant_bits,
            array: Vec::new(),
        };
        for value in lhs.array.iter() {
            if rhs.contains(value) {
                res.array.push(*value);
            }
        }
        Self::ContainerTypeArray(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut a = RoaringBitmap::new();
        let mut b = RoaringBitmap::new();
        for i in 0..5000 {
            a.add(i);
        }
        for i in 5000..10_000 {
            b.add(i);
        }
        for i in 0..5000 {
            assert_eq!(a.contains(i), true);
        }
        for i in 5000..10_000 {
            assert_eq!(a.contains(i), false);
        }
        for i in 0..5000 {
            a.remove(i);
        }
        for i in 0..5000 {
            assert_eq!(a.contains(i), false);
        }
    }

    #[test]
    fn union_same_msb_containers() {
        let mut a = RoaringBitmap::new();
        let mut b = RoaringBitmap::new();
        for i in 0..5000 {
            a.add(i);
        }
        for i in 5000..10_000 {
            b.add(i);
        }

        let c = a.union(&b);
        for i in 0..10_000 {
            assert_eq!(c.contains(i), true);
        }
    }

    #[test]
    fn union_different_msb_containers() {
        let mut a = RoaringBitmap::new();
        let mut b = RoaringBitmap::new();
        for i in 0..5000 {
            a.add(i);
        }
        for i in 5_00_000..10_00_000 {
            b.add(i);
        }

        let c = a.union(&b);
        for i in 0..5000 {
            assert_eq!(c.contains(i), true);
        }
        for i in 5000..5_00_000 {
            assert_eq!(c.contains(i), false);
        }
        for i in 5_00_000..10_00_000 {
            assert_eq!(c.contains(i), true);
        }
    }

    #[test]
    fn intersection_non_empty() {
        let mut a = RoaringBitmap::new();
        let mut b = RoaringBitmap::new();
        for i in 0..5000 {
            a.add(i);
        }
        for i in 2500..7500 {
            b.add(i);
        }

        let c = a.intersection(&b);
        for i in 0..2500 {
            assert_eq!(c.contains(i), false);
        }
        for i in 2500..5000 {
            assert_eq!(c.contains(i), true);
        }
        for i in 5000..7500 {
            assert_eq!(c.contains(i), false);
        }
    }

    #[test]
    fn intersection_empty() {
        let mut a = RoaringBitmap::new();
        let mut b = RoaringBitmap::new();

        for i in 0..5000 {
            a.add(i);
        }
        for  i in 10_00_000..15_00_000 {
            b.add(i);
        }

        let c = a.intersection(&b);
        assert_eq!(c.is_empty(), true);
    }
}
