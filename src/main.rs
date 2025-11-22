use rayon::iter::{IntoParallelIterator, ParallelIterator};
use zeta::{Block, Database};

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [99]))]
fn main() {
    let mut database = Database::default();
    /*let mut blocks: Vec<Block> = (0..1000).into_par_iter().map(|_| {
        let mut block = Block::default();
        for _ in 0..8192 {
            block.insert(String::from(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus suscipit justo a magna dapibus, in porta ipsum auctor. Proin in ornare est. Vivamus vestibulum felis orci, at mattis nisl consequat in. Sed laoreet pretium urna, id volutpat libero vulputate ac. Aliquam tempus ex ac dolor dignissim ornare. Nullam vel nisl leo. Pellentesque sed justo tortor. Donec id quam arcu.",
            ));
        }

        block
    }).collect();

    let mut last = Block::default();
    last.insert(String::from("Arthur"));

    blocks.push(last);

    for block in blocks {
        database.insert(block);
    }

    database.save().unwrap();*/

    database.load().unwrap();

    println!("loaded");
    println!("{:?}", database.get("Arthur"));
}
