pub trait BinarySource: Sized {
    fn get(&self) -> bool;

    fn set_inverted(self, inverted: bool) -> Inverter<Self> {
        Inverter::new(self, inverted)
    }
}
pub trait InvertibleBinarySource: BinarySource {
    fn set_inverted(&mut self, inverted: bool);
}

impl BinarySource for embassy_rp::gpio::Input<'static> {
    fn get(&self) -> bool {
        self.get_level().into()
    }
}
impl InvertibleBinarySource for embassy_rp::gpio::Input<'static> {
    fn set_inverted(&mut self, inverted: bool) {
        self.set_inversion(inverted);
    }
}

/// Wraps a stream and allows its output to be inverted. Handy for input pins that have pull up resistors.
pub struct Inverter<T>
where
    T: BinarySource,
{
    inner: T,
    inverted: bool,
}
impl<T> Inverter<T>
where
    T: BinarySource,
{
    pub fn new(inner: T, inverted: bool) -> Self {
        Self { inner, inverted }
    }
}
impl<T> InvertibleBinarySource for Inverter<T>
where
    T: BinarySource,
{
    fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
    }
}
impl<T> BinarySource for Inverter<T>
where
    T: BinarySource,
{
    fn get(&self) -> bool {
        self.inner.get() ^ self.inverted
    }
}
