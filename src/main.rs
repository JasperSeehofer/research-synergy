mod data_aggregation;
mod datamodels;
use datamodels::paper::Paper;

fn main() {
    let arxivs = data_aggregation::arxiv_api::get_papers();
    for arxiv in arxivs.iter() {
        println!("{:?}", arxiv.title);
    }
    let paper: Paper = Paper::from_arxiv_paper(&arxivs[0]);

    println!("{}", paper);
}
