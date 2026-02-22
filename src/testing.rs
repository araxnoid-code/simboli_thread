pub trait MyOuputTrait {
    type Output;
    fn get_value(self) -> Self::Output;
}

pub trait MyTestingParams {
    fn exec(self) -> impl MyOuputTrait;
}

pub struct MyTesting {}

impl MyTesting {
    pub fn init() -> MyTesting {
        Self {}
    }

    pub fn exec_the_fn<F>(&self, f: F) -> impl MyOuputTrait
    where
        F: MyTestingParams,
    {
        f.exec()
    }
}
