//! containers for id segments

use std::fmt;

/// container for storing id segments
///
/// wrapper around an array with a fixed size
#[derive(Clone, Debug)]
pub struct Segments<T, const N: usize>([T; N]);

impl<T, const N: usize> Segments<T, N> {
    /// references inner array
    pub fn inner(&self) -> &[T; N] {
        &self.0
    }

    /// returns inner array
    pub fn into_inner(self) -> [T; N] {
        self.0
    }
}

impl<T, const N: usize> std::ops::Index<usize> for Segments<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

impl<T, const N: usize> From<[T; N]> for Segments<T, N> {
    fn from(v: [T; N]) -> Self {
        Self(v)
    }
}

impl<T, const N: usize> From<Segments<T, N>> for [T; N] {
    fn from(seg: Segments<T, N>) -> [T; N] {
        seg.0
    }
}

impl<T> Segments<T, 1> {
    /// creates container from 1 segment
    pub fn from_parts(p: T) -> Self {
        Self([p])
    }

    /// references the primary (first) segment
    pub fn primary(&self) -> &T {
        &self.0[0]
    }
}

impl<T> From<T> for Segments<T, 1> {
    fn from(v: T) -> Self {
        Self([v])
    }
}

impl<T> Segments<T, 2> {
    /// creates container from 2 segments
    pub fn from_parts(p: T, s: T) -> Self {
        Self([p, s])
    }

    /// references the primary (first) segment
    pub fn primary(&self) -> &T {
        &self.0[0]
    }

    /// references the secondary (second) segment
    pub fn secondary(&self) -> &T {
        &self.0[1]
    }
}

impl<T> From<(T, T)> for Segments<T, 2> {
    fn from(v: (T, T)) -> Self {
        Self([v.0, v.1])
    }
}

impl<T> Segments<T, 3> {
    /// creates container from 3 segments
    pub fn from_parts(p: T, s: T, t: T) -> Self {
        Self([p, s, t])
    }

    /// references the primary (first) segment
    pub fn primary(&self) -> &T {
        &self.0[0]
    }

    /// references the secondary (second) segment
    pub fn secondary(&self) -> &T {
        &self.0[1]
    }

    /// references the tertiary (third) segment
    pub fn tertiary(&self) -> &T {
        &self.0[2]
    }
}

impl<T> From<(T, T, T)> for Segments<T, 3> {
    fn from(v: (T, T, T)) -> Self {
        Self([v.0, v.1, v.2])
    }
}

impl<T, const N: usize> fmt::Display for Segments<T, N>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;

        for i in 0..N {
            if i != 0 {
                write!(f, ",")?;
            }

            write!(f, "{}", self.0[i])?;
        }

        Ok(())
    }
}

