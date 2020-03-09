/// Helper trait for combining tuples of Content refs
pub trait Combine<Other> {
    type Result;

    fn combine(self, other: Other) -> Self::Result;
}

impl<'x, X> Combine<&'x X> for () {
    type Result = &'x X;

    fn combine(self, other: &'x X) -> Self::Result {
        other
    }
}

impl<'x, 'a, X, A> Combine<&'x X> for &'a A {
    type Result = (&'a A, &'x X);

    fn combine(self, other: &'x X) -> Self::Result {
        (self, other)
    }
}

impl<'x, 'a, 'b, X, A, B> Combine<&'x X> for (&'a A, &'b B) {
    type Result = (&'a A, &'b B, &'x X);

    fn combine(self, other: &'x X) -> Self::Result {
        (self.0, self.1, other)
    }
}

impl<'x, 'a, 'b, 'c, X, A, B, C> Combine<&'x X> for (&'a A, &'b B, &'c C) {
    type Result = (&'a A, &'b B, &'c C, &'x X);

    fn combine(self, other: &'x X) -> Self::Result {
        (self.0, self.1, self.2, other)
    }
}

impl<'x, 'a, 'b, 'c, 'd, X, A, B, C, D> Combine<&'x X> for (&'a A, &'b B, &'c C, &'d D) {
    type Result = (&'b B, &'c C, &'d D, &'x X);

    fn combine(self, other: &'x X) -> Self::Result {
        (self.1, self.2, self.3, other)
    }
}