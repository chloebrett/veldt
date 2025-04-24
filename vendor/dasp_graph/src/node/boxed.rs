use crate::{Buffer, Input, Node};
use core::fmt;
use core::ops::{Deref, DerefMut};

/// A wrapper around a `Box<dyn Node>`.
///
/// Provides the necessary `Sized` implementation to allow for compatibility with the graph process
/// function.
pub struct BoxedNode<P>(pub Box<dyn Node<P>>);

/// A wrapper around a `Box<dyn Node>`.
///
/// Provides the necessary `Sized` implementation to allow for compatibility with the graph process
/// function.
///
/// Useful when the ability to send nodes from one thread to another is required. E.g. this is
/// common when initialising nodes or the audio graph itself on one thread before sending them to
/// the audio thread.
pub struct BoxedNodeSend<P>(pub Box<dyn Node<P> + Send>);

impl<P> BoxedNode<P> {
    /// Create a new `BoxedNode` around the given `node`.
    ///
    /// This is short-hand for `BoxedNode::from(Box::new(node))`.
    pub fn new<T>(node: T) -> Self
    where
        T: 'static + Node<P>,
    {
        Self::from(Box::new(node))
    }
}

impl<P> BoxedNodeSend<P> {
    /// Create a new `BoxedNode` around the given `node`.
    ///
    /// This is short-hand for `BoxedNode::from(Box::new(node))`.
    pub fn new<T>(node: T) -> Self
    where
        T: 'static + Node<P> + Send,
    {
        Self::from(Box::new(node))
    }
}

impl<P> Node<P> for BoxedNode<P> {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &P) {
        self.0.process(inputs, output, payload)
    }
}

impl<P> Node<P> for BoxedNodeSend<P> {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &P) {
        self.0.process(inputs, output, payload)
    }
}

impl<T, P> From<Box<T>> for BoxedNode<P>
where
    T: 'static + Node<P>,
{
    fn from(n: Box<T>) -> Self {
        BoxedNode(n as Box<dyn Node<P>>)
    }
}

impl<T, P> From<Box<T>> for BoxedNodeSend<P>
where
    T: 'static + Node<P> + Send,
{
    fn from(n: Box<T>) -> Self {
        BoxedNodeSend(n as Box<dyn Node<P> + Send>)
    }
}

impl<P> Into<Box<dyn Node<P>>> for BoxedNode<P> {
    fn into(self) -> Box<dyn Node<P>> {
        self.0
    }
}

impl<P> Into<Box<dyn Node<P> + Send>> for BoxedNodeSend<P> {
    fn into(self) -> Box<dyn Node<P> + Send> {
        self.0
    }
}

impl<P> fmt::Debug for BoxedNode<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BoxedNode").finish()
    }
}

impl<P> fmt::Debug for BoxedNodeSend<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BoxedNodeSend").finish()
    }
}

impl<P> Deref for BoxedNode<P> {
    type Target = Box<dyn Node<P>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> Deref for BoxedNodeSend<P> {
    type Target = Box<dyn Node<P> + Send>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> DerefMut for BoxedNode<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P> DerefMut for BoxedNodeSend<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
