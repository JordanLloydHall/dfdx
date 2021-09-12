use crate::gradients::{ops::GradientRef, traits::Params, Grad, GradientTape};
use crate::randomize::Randomize;
use ndarray::{Array, Dimension, ShapeBuilder};

pub trait ShapedArray {
    type Dimension: Dimension;
    type Shape: ShapeBuilder<Dim = Self::Dimension>;
    const SHAPE: Self::Shape;
    const NUM_ELEMENTS: usize;

    fn data(&self) -> &Array<f32, Self::Dimension>;
    fn mut_data(&mut self) -> &mut Array<f32, Self::Dimension>;
}

pub trait Activations {
    fn relu(&mut self) -> Self;
    fn sin(&mut self) -> Self;
    fn cos(&mut self) -> Self;
    fn ln(&mut self) -> Self;
    fn exp(&mut self) -> Self;
    fn sigmoid(&mut self) -> Self;
    fn tanh(&mut self) -> Self;
    fn square(&mut self) -> Self;
}

pub trait Tensor: Randomize + Params + Default + ShapedArray + Activations {
    fn with_grad(data: Array<f32, Self::Dimension>, grad: Option<Grad>) -> Self;

    fn grad(&self) -> &Option<Grad>;
    fn mut_grad(&mut self) -> &mut Option<Grad>;

    fn gradient_ref(&self) -> GradientRef {
        self.grad().as_ref().unwrap().gradient_ref
    }

    fn take_tape(&mut self) -> Option<Box<GradientTape>> {
        self.mut_grad()
            .as_mut()
            .map(|grad| grad.tape.take())
            .flatten()
    }

    fn backward(&mut self) -> Option<Box<GradientTape>> {
        self.mut_grad().as_mut().map(|grad| {
            let mut tape = grad.tape.take().unwrap();
            tape.backward(grad.gradient_ref);
            tape
        })
    }

    fn keep_tape(&mut self, mut tape: Box<GradientTape>) {
        let grad = self
            .mut_grad()
            .get_or_insert_with(|| Grad::new(tape.store_gradient(Self::SHAPE)));
        grad.tape = Some(tape);
    }
}

pub trait Batch {
    type Batched<const B: usize>: Tensor;
}