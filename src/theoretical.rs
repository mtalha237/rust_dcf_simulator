/*
    p = 1- ((1-tao)^(n-1))
    tao = 2 / (1 + CWmin + p * CWmin * (1 - ((2*p) ^ m) )/(1-2p))
 */


pub fn calculate_tao_and_p (num_nodes: usize, cw_min: usize, max_mul: i32) -> (f64, f64) {
    let mut p_current = 0.5;
    let mut p_diff = 0.9;
    let mut tao: f64 = 0.0;
    while p_diff > 0.0001 {
        let denom: f64 = {
            if p_current != 0.5 {
                let denom: f64 = 1.0f64 - ((2.0f64 * p_current).powi(max_mul));
                let denom: f64 = denom / (1.0 - 2.0*p_current);
                let denom: f64 = 1.0 + (cw_min as f64) + p_current * (cw_min as f64) * denom;
                denom
            } else {
                let denom: f64 = 1.0 + (cw_min as f64) +  (max_mul as f64) * (cw_min as f64) * 0.5;
                denom
            }
        };
        
        tao = 2.0 / (denom);
        let p_next: f64 = 1.0 - ((1.0 - tao).powf((num_nodes as f64) - 1.0));
        p_diff = (p_current - p_next).abs();
        p_current = p_next;
    }
    println!("Thoeretical calculations: p_success: {}, tao: {}", 1.0 - p_current, tao);
    (tao, 1.0 - p_current)
}
