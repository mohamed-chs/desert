use std::cmp::{max as std_max};

trait DesertMatMul<Rhs> {
    type Output;
    fn desert_matmul(self, rhs: Rhs) -> Self::Output;
}

impl DesertMatMul<Vec<f32>> for Vec<f32> {
    type Output = Vec<f32>;

    fn desert_matmul(self, rhs: Vec<f32>) -> Self::Output {
        vec![self.into_iter().zip(rhs).map(|(a, b)| a * b).sum()]
    }
}

impl DesertMatMul<Vec<f32>> for Vec<Vec<f32>> {
    type Output = Vec<f32>;

    fn desert_matmul(self, rhs: Vec<f32>) -> Self::Output {
        self.into_iter()
            .map(|row| row.into_iter().zip(rhs.iter().copied()).map(|(a, b)| a * b).sum())
            .collect()
    }
}

fn desert_matmul<L, R>(lhs: L, rhs: R) -> <L as DesertMatMul<R>>::Output
where
    L: DesertMatMul<R>,
{
    lhs.desert_matmul(rhs)
}

trait Stats {
    fn bump(&mut self, delta: i32) -> i32 {
        return 0;
    }
}
struct Counter {
    pub value: i32,
}
impl Stats for Counter {
    fn bump(&mut self, delta: i32) -> i32 {
        self.value = self.value + delta;
        return self.value;
    }
}
fn dot_sum(xs: Vec<f32>, ys: Vec<f32>) -> f32 {
    let out = desert_matmul((xs).clone(), (ys).clone());
    return out[0];
}
fn consume_head(items: &mut Vec<i32>) -> i32 {
    return std::mem::take(&mut items[0]);
}
fn main() {
    let mut counter = Counter { value: 2 };
    let bumped = counter.bump(std_max(3, 4));
    let mut data = vec![9, 8, 7];
    let head = consume_head(&mut data);
    let score = dot_sum(vec![1.0, 2.0], vec![3.0, 4.0]);
    #[allow(unreachable_patterns)]
    match head {
        9 => {
            println!("{}", format!("head {:?} bumped {:?} score {:?}", head, bumped, score));
        }
        0 => {
            println!("{}", "zero".to_string());
        }
        _ => {
            panic!("non-exhaustive match in Desert source");
        }
    }
}

