use crate::prelude::*;

pub fn value_mask<T: Tensor>(t: T, other: &T::NoTape, value: f32) -> T {
    let result =
        T::NoTape::new(t.data().zip_map(
            other.data(),
            |&a, b| {
                if b.eq(&value) {
                    value
                } else {
                    a
                }
            },
        ));
    let (t, mut tape_holder) = t.split_tape_holder();
    let deriv = other
        .data()
        .map_elems(|v| if v.eq(&value) { 0.0 } else { 1.0 });
    let _t = t.phantom();
    let _result = result.phantom();
    tape_holder.add_operation(move |tape| {
        let d_grad = deriv.mul(tape.gradient(&_result));
        tape.mut_gradient(&_t).add_assign(&d_grad);
    });
    result.with_tape_holder(tape_holder)
}

macro_rules! tensor_impl {
    ($typename:ident, [$($Vs:tt),*]) => {
impl<$(const $Vs: usize, )* H: TapeHolder> $typename<$($Vs, )* H> {
    pub fn value_mask(self, mask: &$typename<$($Vs, )* NoTape>, value: f32) -> Self {
        value_mask(self, mask, value)
    }
}
    };
}

tensor_impl!(Tensor0D, []);
tensor_impl!(Tensor1D, [M]);
tensor_impl!(Tensor2D, [M, N]);
tensor_impl!(Tensor3D, [M, N, O]);
tensor_impl!(Tensor4D, [M, N, O, P]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_0d() {
        let t = Tensor0D::new(1.0);
        let m = Tensor0D::new(-1e10);
        let r = t.with_tape().value_mask(&m, -1e10);
        assert_eq!(r.data(), &-1e10);
        let gradients = backward(r.mean());
        assert_eq!(gradients.gradient(&t), &0.0);
    }

    #[test]
    fn test_mask_1d() {
        let t: Tensor1D<3> = Tensor1D::new([1.0, 2.0, 3.0]);
        let m: Tensor1D<3> = Tensor1D::new([-1e10, 0.0, -1e10]);
        let r = t.with_tape().value_mask(&m, -1e10);
        assert_eq!(r.data(), &[-1e10, 2.0, -1e10]);
        let gradients = backward(r.mean());
        assert_eq!(gradients.gradient(&t), &[0.0, 1.0 / 3.0, 0.0]);
    }

    #[test]
    fn test_mask_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let m: Tensor2D<2, 3> = Tensor2D::new([[-1e10, 0.0, -1e10], [1.0, -1e10, -1e9]]);
        let r = t.with_tape().value_mask(&m, -1e10);
        assert_eq!(r.data(), &[[-1e10, 2.0, -1e10], [4.0, -1e10, 6.0]]);
        let gradients = backward(r.mean());
        assert_eq!(
            gradients.gradient(&t),
            &[[0.0, 1.0 / 6.0, 0.0], [1.0 / 6.0, 0.0, 1.0 / 6.0]]
        );
    }

    #[test]
    fn test_mask_3d() {
        todo!();
    }
}