extern crate rand;

use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

pub fn create_random_secret(secret_len: usize) -> String {
    let mut rng = rand::thread_rng();

    (0..secret_len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_random_secret() {
        let len = 30;
        let sec1 = create_random_secret(len);
        let sec2 = create_random_secret(len);
        assert_eq!(sec1.len(), 30);
        assert_eq!(sec2.len(), 30);
        assert_ne!(sec2, sec1);

        let len = 47;
        assert_eq!(len, create_random_secret(len).len())
    }
}
