use constants::*;
use phone;

pub fn prob(spectrum: &[f64; N_DIMENSION], state: &phone::State) -> f64 {
    let probs_for_pdf: Vec<f64> = state.pdfs.iter()
        .map(|pdf| prob_for_pdf(spectrum, pdf))
        .collect();
    
    sum_without_underflow(&probs_for_pdf)
}

fn sum_without_underflow(log_values: &[f64]) -> f64 {
    let first_log_value = log_values[0];
    let mut sum = 0f64;
    for log_value in log_values {
        sum += (log_value - first_log_value).exp();
    }
    first_log_value + sum.ln()
}

fn prob_for_pdf(spectrum: &[f64; N_DIMENSION], pdf: &phone::Pdf) -> f64 {
    let mut sum = 0f64;
    for d in 0..N_DIMENSION {
        sum -= pdf.var[d].ln() / 2f64 + (spectrum[d] - pdf.mean[d]) * (spectrum[d]- pdf.mean[d]) / (2f64 * pdf.var[d]);
    }
    sum + pdf.weight.ln()
}
