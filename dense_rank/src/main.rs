// core algorithm used in `dense_rank` sql function 
// https://www.sqlitetutorial.net/sqlite-window-functions/sqlite-dense_rank/
fn main() {
   let mut numbers = std::fs::read_to_string("input.txt")
       .expect("input.txt")
       .lines()
       .map(|line| line.parse::<i32>().unwrap())
       .collect::<Vec<i32>>();

   numbers.sort();
   
   let mut ranks = Vec::with_capacity(numbers.len());
   let mut rank = 1;

   let mut p = i32::MAX;
   for n in &numbers {
        if p != i32::MAX && p != *n {
            rank += 1;
        }
       p = *n;
       ranks.push(rank);
   }
   println!("Number Rank");
   for (n, r) in numbers.iter().zip(ranks.iter()) {
       println!("{} {}", n, r);
   }
}
