#[cfg(not(feature = "std"))]
extern crate libm;

#[inline]
/// Returns the smallest integer greater than or equal to `x`.
pub(crate) fn ceil(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.ceil()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::ceil(x)
    }
}

#[inline]
/// Computes `2.0^x`.
pub(crate) fn exp2(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.exp2()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::pow(2.0, x)
    }
}

#[inline]
/// Returns the largest integer less than or equal to `x`.
pub(crate) fn floor(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::floor(x)
    }
}


#[inline]
/// Linearly interpolates between `a` and `b` by `t`, where `t` is in the range `[0.0, 1.0]`.
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

#[inline]
/// Computes `log(x, base)`.
pub(crate) fn log(x: f64, base: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        f64::log(x, base)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::log(x) / libm::log(base)
    }
}

#[inline]
/// Computes `x^y`.
pub(crate) fn powf(x: f64, y: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.powf(y)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::pow(x, y)
    }
}

#[inline]
/// Computes the square root of `x`.
pub(crate) fn sqrt(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sqrt()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::sqrt(x)
    }
}
