type BufRange = std::ops::Range<usize>;


const PATH_PARAMS_LIMIT: usize = 4;
pub(crate) struct PathParams {
    params: [Option<BufRange>; PATH_PARAMS_LIMIT],
    next:   usize,
}
impl PathParams {
    
}

