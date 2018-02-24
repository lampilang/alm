use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
    Nil(),
    Byte(u8),
    IWord(isize),
    UWord(usize),
    I64(i64),
    U64(u64),
    F64(f64),
    Array(Array),
    Tuple(Tuple),
    Function(Function),
    Extension(*mut ()),
}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Nil(usize),
    Byte(Arc<Vec<u8>>),
    IWord(Arc<Vec<isize>>),
    UWord(Arc<Vec<usize>>),
    I64(Arc<Vec<i64>>),
    U64(Arc<Vec<u64>>),
    F64(Arc<Vec<f64>>),
    Array(Arc<Vec<Array>>),
    Tuple(Arc<Vec<Tuple>>),
    Function(Arc<Vec<Function>>),
    Extension(Arc<Vec<*mut ()>>),
}

#[derive(Clone, Debug)]
pub struct Array {
    memory: Arc<ArrayInner>
}

#[derive(Clone, Debug)]
pub struct Tuple {
    memory: Arc<TupleInner>
}

#[derive(Clone, Debug)]
pub struct Function {
    memory: Arc<FunctionInner>
}

#[derive(Clone, Debug)]
struct FunctionInner {
    env: Value,
    bc: Arc<Vec<u8>>,
    name: Arc<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct ArrayInner {
    typ: ArrayType,
}

#[derive(Clone, Debug)]
struct TupleInner {
    elems: Vec<Value>,
}
