use paste::paste;

use std::mem::MaybeUninit;

macro_rules! impl_shotgun_intersection_iterator {
    ($len:literal, $len_sub_one:literal, $($n:literal),*) => {
        paste !{
            pub fn [<shotgun_intersect $len>]<'a, T: Ord>(a: &'a [T], b: &'a [T]) -> impl Iterator<Item = &'a T> {
                let (shorter, longer) = if a.len() < b.len() { (a, b) } else { (b, a) };
                [<ShotgunIntersectionIterator $len>] {
                    shorter,
                    longer,
                    i_s: 0,
                    i_l: 0,
                    buf: MaybeUninit::uninit(),
                    buf_len: 0,
                }
            }

            pub struct [<ShotgunIntersectionIterator $len>]<'a, T> {
                shorter: &'a [T],
                longer: &'a [T],
                i_s: usize,
                i_l: usize,
                buf: MaybeUninit<[&'a T; $len]>,
                buf_len: usize,
            }

            impl<'a, T: Ord> [<ShotgunIntersectionIterator $len>]<'a, T> {
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

            impl<'a, T: Ord> Iterator for [<ShotgunIntersectionIterator $len>]<'a, T> {
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

                    while idx_s + $len <= len_s && idx_l < len_l {
                        let mut n = len_l - idx_l;
                        if n == 0 {
                            return None;
                        }

                        $(
                            let [<target_ $n>] = &shorter[idx_s + $n];
                            let mut [<base_ $n>] = idx_l;
                        )*

                        while n > 1 {
                            let half = n >> 1;
                            $(
                                if longer[[<base_ $n>] + half] < *[<target_ $n>]  {
                                    [<base_ $n>] += half;
                                }
                            )*
                            n -= half;
                        }

                        $(
                            let [<index_ $n>] = [<base_ $n>] + (longer[[<base_ $n>]] < *[<target_ $n>]) as usize;
                            if [<index_ $n>] < len_l && longer[[<index_ $n>]] == *[<target_ $n>] {
                                self.push(&shorter[idx_s + $n]);
                            }
                        )*

                        idx_s += $len;
                        idx_l = [<index_ $len_sub_one>];
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
        }
    };
}

impl_shotgun_intersection_iterator!(4, 3, 3, 2, 1, 0);
impl_shotgun_intersection_iterator!(8, 7, 7, 6, 5, 4, 3, 2, 1, 0);
impl_shotgun_intersection_iterator!(16, 15, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
impl_shotgun_intersection_iterator!(
    32, 31, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
    9, 8, 7, 6, 5, 4, 3, 2, 1, 0
);

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
