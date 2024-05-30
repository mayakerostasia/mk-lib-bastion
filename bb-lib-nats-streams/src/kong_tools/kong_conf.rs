struct Funk<T,O>(fn(T) -> O);

struct KongConf<T, O> {
    subject: String,
    funk_name: String,
    funk: Funk<T,O> ,
}
