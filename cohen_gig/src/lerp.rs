/// Types that support linear interpolation.
pub trait Lerp {
    /// Linearly interpolate from self towards `other` by the given amount.
    fn lerp(&self, other: &Self, amt: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, amt: f32) -> Self {
        *self + (*other - *self) * amt
    }
}

impl Lerp for [f32; 3] {
    fn lerp(&self, other: &Self, amt: f32) -> Self {
        let [ax, ay, az] = *self;
        let [bx, by, bz] = other;
        [ax.lerp(bx, amt), ay.lerp(by, amt), az.lerp(bz, amt)]
    }
}

use nannou::color::LinSrgb;
use nannou::color::lin_srgb;
impl Lerp for LinSrgb {
    fn lerp(&self, other: &Self, amt: f32) -> Self {
        let (ax, ay, az) = self.into_components();
        let (bx, by, bz) = &other.into_components();
        lin_srgb(ax.lerp(bx, amt), ay.lerp(by, amt), az.lerp(bz, amt))
    }
}