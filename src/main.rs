use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::ops::Deref;
use std::path::Display;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fmt, sync, thread};

/***************************************************************************************************
 * test trait !
 **************************************************************************************************/

trait DemoTrait {
    fn show(&self);
}

struct Foo {}

impl DemoTrait for Foo {
    fn show(&self) {
        println!("Foo");
    }
}

fn testTraitAsArg(arg: &impl DemoTrait) {
    arg.show();
}

fn testTraitGenericConstraints0<T: DemoTrait>(arg: &T) {
    arg.show();
}

fn testTraitGenericConstraints1<T>(arg: &T)
where
    T: DemoTrait,
{
    arg.show();
}

fn testTraitAsRet() -> impl DemoTrait {
    let foo_obj = Foo {};
    return foo_obj;
}

fn test_trait() {
    let foo_obj = Foo {};
    testTraitAsArg(&foo_obj);
    testTraitGenericConstraints0(&foo_obj);
    testTraitGenericConstraints1(&foo_obj);
    let obj = testTraitAsRet();
    obj.show()
}

/***************************************************************************************************
 * test lift time !
 **************************************************************************************************/

// life time test
struct Bar<'a> {
    sref: &'a str,
}

impl<'b> Bar<'b> {
    fn show(&self) -> &str {
        println!("#show: {}", self.sref);
        self.sref
    }
}

fn test_struct_prop_life_time() {
    let bar = Bar { sref: "xyz" };
    println!("bar sref : {}", bar.sref);
    bar.show();
}

// test node list
struct SNode<'a> {
    val: u32,
    next: Option<&'a SNode<'a>>, // 指向下一阶段的引用
}

impl<'a> SNode<'a> {
    fn show(&self) {
        let mut curr = self;
        loop {
            println!("--> {}", curr.val);
            match curr.next {
                None => break,
                Some(node) => curr = node,
            }
        }
    }
}

fn test_node_list() {
    let mut sn0 = SNode { val: 0, next: None };
    let mut sn1 = SNode { val: 1, next: None };
    let sn2 = SNode { val: 2, next: None };
    sn1.next = Some(&sn2);
    sn0.next = Some(&sn1);

    sn0.show();
}

/***************************************************************************************************
 * test raw pointer share between thread!
 **************************************************************************************************/
struct PointerWrapper {
    pointer: *mut u32,
}

impl PointerWrapper {
    pub fn new(pointer: *mut u32) -> Self {
        Self { pointer }
    }
    pub fn get(&self) -> *mut u32 {
        self.pointer
    }
}

unsafe impl Send for PointerWrapper {}

fn test_raw_pointer() {
    let mut value = 1u32;
    let mut mutable_ptr: *mut u32 = &mut value;
    let wrapper = PointerWrapper::new(mutable_ptr);
    let arc_wrapper = Arc::new(sync::Mutex::new(wrapper));

    let mutex_wrapper = arc_wrapper.clone();

    let foo = thread::spawn(move || {
        unsafe {
            let wrapper = mutex_wrapper;
            let pointer = wrapper.lock().unwrap().pointer;
            println!("[1]--{}", *pointer);
            *pointer = 2;
            println!("[1]--{}", *pointer);
            // 互斥锁在作用于结束时, 自动释放锁
        }
    });

    let mutex_wrapper = arc_wrapper.clone();
    let bar = thread::spawn(move || unsafe {
        let wrapper = mutex_wrapper;
        let pointer = wrapper.lock().unwrap().pointer;
        println!("[2]--{}", *pointer);
    });

    bar.join();
    foo.join();
}

/***************************************************************************************************
 * test function generic
 **************************************************************************************************/
fn largest<T: Copy + PartialOrd>(list: &[T]) -> T {
    let mut largest = list[0].clone();

    for &item in list.iter() {
        if item > largest {
            largest = item.clone();
        }
    }

    largest
}

pub fn test_largest() {
    let num_array: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let largest = largest(&num_array);
    println!("# largest value: {}", largest)
}

/***************************************************************************************************
 * test struct generic
 **************************************************************************************************/
struct Pointer<T> {
    x: T,
    y: T,
}

impl<T: fmt::Display> fmt::Display for Pointer<T> {
    // * 第一个T说明第二T是泛型，而不是定义的普通类型
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "x: {}, y: {}", self.x, self.y)
    }
}

fn test_pointer() {
    let pointer = Pointer { x: 1, y: 2 };
    println!("pointer: {}", pointer)
}

/***************************************************************************************************
 * test enum generic
 **************************************************************************************************/
enum Action<F> {
    Sleep,
    Run,
    Walk,
    Eat(F),
}

fn test_action() {
    let action = Action::Eat("rice"); // 编译时，F类型替换为字符串
    match action {
        Action::Eat(food) => println!("# food = {}", food),
        _ => println!("other action!"),
    }

    let action = Action::Eat(100); // 编译时, F类型替换为整数
    match action {
        Action::Eat(food) => println!("# food = {}", food),
        _ => println!("other action!"),
    }
}

/***************************************************************************************************
 * test deref
 **************************************************************************************************/
struct Alpha<T> {
    val: T,
}

impl Alpha<i32> {
    fn inc(&mut self) {
        self.val = self.val + 1;
    }
}

impl<T> Deref for Alpha<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

fn test_deref() {
    let a = Alpha { val: 1 };
    println!("value0 = {}", a.val);
    println!("value1 = {}", *a);
}

/***************************************************************************************************
 * test drop
 **************************************************************************************************/
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

fn test_drop() {
    let c = CustomSmartPointer {
        data: String::from("my stuff"),
    };
    let d = CustomSmartPointer {
        data: String::from(" stuff"),
    };
    println!("CustomSmartPointers created.")
}

/***************************************************************************************************
 * test error handle
 **************************************************************************************************/
use std::fs::File;

// unwrap方法从Option或Result中获取值, 并返回值。如果调用unwrap时，值为None或者为Err，则触发panic
fn test_err_unwarp() {
    let f1 = File::open("hello.txt").unwrap();
}

// expect方法和unwrap类似, 从Option或Result中获取值, 并返回值。不同的是, expect方法允许定义panic的错误消息
fn test_err_expect() {
    let f2 = File::open("hello.txt").expect("not found!");
}

fn func0() -> Result<File, bool> {
    let file = File::open("hello.txt");
    match file {
        Ok(f) => Ok(f),
        Err(e) => {
            println!("# error: {}", e);
            Err(false)
        }
    }
}

fn func1() -> Result<File, bool> {
    let file = func0()?; // 如果返回错误，则直接把error传递出去
    Ok(file)
}

fn test_err_pass() {
    let file = func1();
    match file {
        Ok(f) => println!("open ok!"),
        Err(e) => println!("{}", e),
    }
}

/***************************************************************************************************
 * test smart pointer
 **************************************************************************************************/
use std::rc::Rc;
fn test_rc() {
    // rc 只能在单线程中使用
    let shared_number = Rc::new(42);
    let ref1 = Rc::clone(&shared_number);

    let ref2 = Rc::clone(&shared_number);
    println!("Reference count:{}", Rc::strong_count(&shared_number))
}

// 1. 在有borrow的情况下，borrow_mut是非法的；2. 只有brorrow_mut, 且多次brorrow_mut也是非法的
fn test_ref_cell() {
    // 创建一个包含可变数据的 RefCell
    let data = RefCell::new(vec![1, 2, 3]);

    // 获取不可变引用，并读取数据
    let shared_reference = data.borrow();
    println!("Shared data: {:?}", *shared_reference);

    // 尝试获取可变引用，并修改数据
    // {
    //     let mut mutable_reference = data.borrow_mut();
    //     mutable_reference.push(4);
    // }

    // 获取不可变引用，并读取修改后的数据
    let shared_reference2 = data.borrow();
    println!("Shared data (after modification): {:?}", *shared_reference2);
}



// Arc: 使用Arc，在多线程访问共享不可变变量
// Arc与Mutex、Arc与Atomic，在多线程访问共享可变变量
fn test_thread_arc() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let ref0 = data.clone();
    let job0 = thread::spawn( move || {
        loop {
            let xx = ref0.lock().unwrap();
            println!("{:?}", xx);
            thread::sleep(Duration::from_secs(1));
        }
    });

    let mut ref1 = data.clone();
    let job1 = thread::spawn( move || {

        loop {
            println!("{:?}", ref1);
            let mut xx = ref1.lock().unwrap();
            xx.push(4);
            thread::sleep(Duration::from_secs(1));
        }
    });
    job1.join();
    job0.join();


}

#[cfg(test)]
mod tests {

    #[test]
    fn test13() {
        super::test_thread_arc()
    }


    #[test]
    fn test11() {
        super::test_ref_cell()
    }

    #[test]
    fn test0() {
        super::test_largest()
    }

    #[test]
    fn test1() {
        super::test_pointer()
    }

    #[test]
    fn test2() {
        super::test_action()
    }

    #[test]
    fn test3() {
        super::test_trait()
    }

    #[test]
    fn test4() {
        super::test_struct_prop_life_time()
    }

    #[test]
    fn test5() {
        super::test_node_list()
    }

    #[test]
    fn test6() {
        super::test_deref()
    }

    #[test]
    fn test7() {
        super::test_err_unwarp()
    }

    #[test]
    fn test8() {
        super::test_err_pass()
    }

    #[test]
    fn test9() {
        super::test_drop()
    }

    #[test]
    fn test10() {
        super::test_rc()
    }
}

fn main() {
    test_drop();
}
