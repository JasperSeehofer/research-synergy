mod arxiv_api;

fn main() {
    let arxivs = arxiv_api::get_papers();
    for arxiv in arxivs.iter() {
        println!("{:?}", arxiv.title);
    }
    println!("Hello World");
}
