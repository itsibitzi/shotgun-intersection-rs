use std::mem::MaybeUninit;

pub fn intersect<'a, T: Ord>(a: &'a [T], b: &'a [T]) -> impl Iterator<Item = &'a T> {
    let (shorter, longer) = if a.len() < b.len() { (a, b) } else { (b, a) };
    ShotgunIntersectionIterator {
        shorter,
        longer,
        i_s: 0,
        i_l: 0,
        buf: MaybeUninit::uninit(),
        buf_len: 0,
    }
}

pub struct ShotgunIntersectionIterator<'a, T> {
    shorter: &'a [T],
    longer: &'a [T],
    i_s: usize,
    i_l: usize,
    buf: MaybeUninit<[&'a T; 4]>,
    buf_len: usize,
}

impl<'a, T: Ord> ShotgunIntersectionIterator<'a, T> {
    fn is_empty(&self) -> bool {
        self.buf_len == 0
    }

    fn push(&mut self, v: &'a T) {
        let buf = unsafe { self.buf.assume_init_mut() };
        buf[self.buf_len] = v;
        self.buf_len += 1;
    }

    fn pop_unchecked(&mut self) -> Option<&'a T> {
        self.buf_len -= 1;
        let item = unsafe { self.buf.assume_init_ref()[self.buf_len] };
        Some(item)
    }
}

impl<'a, T: Ord> Iterator for ShotgunIntersectionIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_empty() {
            return self.pop_unchecked();
        }

        let shorter = self.shorter;
        let longer = self.longer;
        let len_s = shorter.len();
        let len_l = longer.len();

        let mut idx_s = self.i_s;
        let mut idx_l = self.i_l;

        if idx_s >= len_s || idx_l >= len_l {
            return None;
        }

        while idx_s + 4 <= len_s && idx_l < len_l {
            let mut n = len_l - idx_l;
            if n == 0 {
                return None;
            }

            let target1 = &shorter[idx_s];
            let target2 = &shorter[idx_s + 1];
            let target3 = &shorter[idx_s + 2];
            let target4 = &shorter[idx_s + 3];

            let mut base1 = idx_l;
            let mut base2 = idx_l;
            let mut base3 = idx_l;
            let mut base4 = idx_l;

            while n > 1 {
                let half = n >> 1;
                if longer[base1 + half] < *target1 {
                    base1 += half;
                }
                if longer[base2 + half] < *target2 {
                    base2 += half;
                }
                if longer[base3 + half] < *target3 {
                    base3 += half;
                }
                if longer[base4 + half] < *target4 {
                    base4 += half;
                }
                n -= half;
            }
            let index1 = base1 + (longer[base1] < *target1) as usize;
            let index2 = base2 + (longer[base2] < *target2) as usize;
            let index3 = base3 + (longer[base3] < *target3) as usize;
            let index4 = base4 + (longer[base4] < *target4) as usize;

            if index4 < len_l && longer[index4] == *target4 {
                self.push(&shorter[idx_s + 3]);
            }
            if index3 < len_l && longer[index3] == *target3 {
                self.push(&shorter[idx_s + 2]);
            }
            if index2 < len_l && longer[index2] == *target2 {
                self.push(&shorter[idx_s + 1]);
            }
            if index1 < len_l && longer[index1] == *target1 {
                self.push(&shorter[idx_s]);
            }
            idx_s += 4;
            idx_l = index4;
            self.i_s = idx_s;
            self.i_l = idx_l;
            if !self.is_empty() {
                return self.pop_unchecked();
            }
        }
        if idx_s < len_s && idx_l < len_l {
            let val_s = &shorter[idx_s];
            self.i_s += 1;
            longer[idx_l..]
                .binary_search(val_s)
                .map(|pos| {
                    self.i_l = idx_l + pos;
                    val_s
                })
                .inspect_err(|pos| self.i_l = idx_l + pos)
                .ok()
        } else {
            None
        }
    }
}

// Vibecoded galloping intersect just for comparison
pub fn galloping_intersect<'a, T: Ord>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
    let (mut i, mut j) = (0, 0);
    let mut result = Vec::new();
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            result.push(&a[i]);
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            // Gallop a
            let mut step = 1;
            while i + step < a.len() && a[i + step] < b[j] {
                step <<= 1;
            }
            let new_i = (i + step).min(a.len());
            i = match a[i..new_i].binary_search(&b[j]) {
                Ok(pos) => i + pos,
                Err(pos) => i + pos,
            };
        } else {
            // Gallop b
            let mut step = 1;
            while j + step < b.len() && b[j + step] < a[i] {
                step <<= 1;
            }
            let new_j = (j + step).min(b.len());
            j = match b[j..new_j].binary_search(&a[i]) {
                Ok(pos) => j + pos,
                Err(pos) => j + pos,
            };
        }
    }
    result
}
