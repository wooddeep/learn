use std::{fmt, sync, thread};
use std::fmt::Formatter;
use std::ops::Deref;
use std::path::Display;
use std::sync::{Arc, Mutex};

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

fn testTraitGenericConstraints1<T>(arg: &T) where T: DemoTrait {
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
                Some(node) => {
                    curr = node
                }
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
    let bar = thread::spawn(move || {
        unsafe {
            let wrapper = mutex_wrapper;
            let pointer = wrapper.lock().unwrap().pointer;
            println!("[2]--{}", *pointer);
        }
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
        Err(e) => println!("{}", e)
    }
}

#[cfg(test)]
mod tests {
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
    fn test3() { super::test_trait() }

    #[test]
    fn test4() { super::test_struct_prop_life_time() }

    #[test]
    fn test5() { super::test_node_list() }

    #[test]
    fn test6() { super::test_deref() }

    #[test]
    fn test7() { super::test_err_unwarp() }

    #[test]
    fn test8() { super::test_err_pass() }
}

fn main() {}