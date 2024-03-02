use std::{mem::{ManuallyDrop, MaybeUninit}, ops::{self, Bound, Deref, Index, IndexMut, RangeBounds}};

#[derive(Debug, Clone)]
struct BinTreeVec(Vec<i32>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Indexer(usize);

impl Index<Indexer> for BinTreeVec {
    type Output = i32;

    fn index(&self, index: Indexer) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<Indexer> for BinTreeVec {
    fn index_mut(&mut self, index: Indexer) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl ops::Add<usize> for Indexer {
    type Output = Indexer;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for Indexer {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for Indexer {
    type Output = Indexer;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl ops::SubAssign<usize> for Indexer {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Indexer {
    const fn child(self) -> (Self, Self) {
        // ((self + 1) * 2 - 1, (self + 1) * 2 + 1 - 1)
        let tmp = (self.0 + 1) * 2;
        (Self(tmp - 1), Self(tmp))
    }

    const fn parent(self) -> Self {
        Self((self.0 + 1) / 2 - 1)
    }

    const fn is_root(self) -> bool {
        self.0 == 0
    }

    const fn is_left(self) -> bool {
        self.0 % 2 == 1
    }

    const fn is_right(self) -> bool {
        self.0 % 2 == 0
    }
}

impl BinTreeVec {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SegTree {
    tree: BinTreeVec,
}

impl SegTree {
    pub fn from_slice(slc: &[i32]) -> Self {
        let len = slc.len();
        let len_aligned = len.next_power_of_two();
        let cap = len_aligned.wrapping_mul(2).wrapping_sub(1);
        let mut v = vec![MaybeUninit::uninit(); cap];
        v[(cap - len_aligned)..].iter_mut().zip(slc.iter().copied().chain(std::iter::repeat(0))).for_each(|(d, s)| {
            d.write(s);
        });
        for i in (0..(cap - len_aligned)).rev() {
            let val = unsafe {
                let (ch1, ch2) = Indexer(i).child();
                v[ch1.0].assume_init_read() + v[ch2.0].assume_init_read()
            };
            v[i].write(val);
        }
        let mut v = ManuallyDrop::new(v);

        Self {
            tree: unsafe {
                BinTreeVec(Vec::from_raw_parts(v.as_mut_ptr() as *mut i32, v.len(), v.capacity()))
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> i32 {
        if self.is_empty() {
            return 0;
        }
        let length = self.tree.len();
        let offset = Indexer(length).parent();
        let mut left = offset + match range.start_bound() {
            Bound::Excluded(&l) => l + 1,
            Bound::Included(&l) => l,
            Bound::Unbounded => 0,
        };
        let mut right = match range.end_bound() {
            Bound::Excluded(&r) => offset + r - 1,
            Bound::Included(&r) => offset + r,
            Bound::Unbounded => Indexer(length - 1),
        };

        let mut result1 = 0;
        let mut result2 = 0;
        while left < right {
            if left.is_right() {
                result1 += self.tree[left];
                left += 1;
            }
            if right.is_left() {
                // result2 = self.tree[right] + result2;
                result2 += self.tree[right];
                right -= 1;
            }
            left = left.parent();
            right = right.parent();
        }

        if left > right {
            debug_assert_eq!(left, right + 1);
            result1 + result2
        } else {
            debug_assert_eq!(left, right);
            result1 + self.tree[left] + result2
        }
    }

    fn evaluate(&mut self, index: Indexer) {
        let val = {
            let (ch1, ch2) = index.child();
            self.tree[ch1] + self.tree[ch2]
        };
        self.tree[index] = val;
    }

    fn get_indexer(&self, index: usize) -> Indexer {
        Indexer(index + self.tree.len() / 2)
    }

    pub fn update(&mut self, index: usize, value: i32) {
        let mut idx = self.get_indexer(index);
        self.tree[idx] = value;
        while !idx.is_root() {
            idx = idx.parent();
            self.evaluate(idx);
        }
    }
}

impl Deref for SegTree {
    type Target = [i32];

    fn deref(&self) -> &Self::Target {
        &self.tree.0[self.get_indexer(0).0..]
    }
}

#[test]
fn from_slice_test() {
    let s = [2, 4, 5, 6, 2];
    let mut segtree = SegTree::from_slice(&s);
    println!("{segtree:?}");
    assert_eq!(11, segtree.query(2..4));
    assert_eq!(9, segtree.query(1..3));
    assert_eq!(17, segtree.query(1..));
    assert_eq!(19, segtree.query(..));

    segtree.update(0, 4);
    println!("{segtree:?}");
    println!("{:?}", &segtree[..]);
}
