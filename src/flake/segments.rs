//! containers for id segments

/// stores two id segments
#[derive(Clone)]
pub struct DualSeg<T>(T, T);

impl<T> DualSeg<T> {
    /// builds the segment from a primary and secondary segments
    pub fn from_parts(primary: T, secondary: T) -> Self {
        DualSeg(primary, secondary)
    }

    /// returns reference to primary segment
    pub fn primary(&self) -> &T {
        &self.0
    }

    /// returns reference to secondary segment
    pub fn secondary(&self) -> &T {
        &self.1
    }

    /// returns tuple of stored segment. `(primary, secondary)`
    pub fn into_parts(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T> From<(T, T)> for DualSeg<T> {
    fn from(tuple: (T, T)) -> Self {
        Self::from_parts(tuple.0, tuple.1)
    }
}
