use zeta::Database;

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [99]))]
fn main() {
    let mut database = Database::default();
    /*for _ in 0..10_000_000 {
        database.insert(String::from(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus suscipit justo a magna dapibus, in porta ipsum auctor. Proin in ornare est. Vivamus vestibulum felis orci, at mattis nisl consequat in. Sed laoreet pretium urna, id volutpat libero vulputate ac. Aliquam tempus ex ac dolor dignissim ornare. Nullam vel nisl leo. Pellentesque sed justo tortor. Donec id quam arcu.",
        ));
    }

    database.insert(String::from(
        "Arthur. Hello consectetur facilisis est quis elementum. Vivamus tincidunt purus ut volutpat scelerisque. Nunc nunc mauris, dictum nec consequat id, molestie vel neque. Mauris suscipit, magna sed dignissim dapibus, erat mi interdum ligula, sit amet consectetur diam mauris quis sapien. Nulla odio mauris, pharetra at magna vel, commodo volutpat leo. Donec vel hendrerit lorem, id porta erat. Morbi ut convallis mi, vel varius felis. In nec felis lacus. Donec vel dui mauris. Phasellus consectetur risus quis viverra auctor. Nam maximus eleifend tellus, ac gravida velit varius et. Sed facilisis ex sit amet metus lobortis, ut venenatis justo scelerisque. Nunc in diam ac magna sagittis tempor vitae eget lectus. Mauris ut odio gravida, feugiat nunc quis, finibus turpis. Hello",
    ));

    database.save().unwrap();*/

    database.load().unwrap();

    // println!("{:?}", database.get("Arthur"));
}
