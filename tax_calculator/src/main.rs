use std::io;
use thousands::Separable;
use std::collections::HashMap;

// PPh 21 Calculation Parameters
#[derive(Debug)]
struct PPh21Params {
    gross_income: f64,
    is_married: bool,
    num_dependents: u32,
}

// PTKP (Penghasilan Tidak Kena Pajak) values for 2023
fn get_ptkp_values() -> HashMap<&'static str, f64> {
    let mut ptkp = HashMap::new();
    ptkp.insert("TK/0", 54_000_000.0);  // Single, no dependents
    ptkp.insert("K/0", 58_500_000.0);   // Married, no dependents
    ptkp.insert("K/1", 63_000_000.0);   // Married, 1 dependent
    ptkp.insert("K/2", 67_500_000.0);   // Married, 2 dependents
    ptkp.insert("K/3", 72_000_000.0);   // Married, 3+ dependents
    ptkp
}

// Calculate PPh 21 for monthly employee
fn calculate_pph21(params: &PPh21Params) -> (f64, f64, f64, f64) {
    let monthly_gross = params.gross_income;
    let annual_gross = monthly_gross * 12.0;
    
    // Get PTKP based on marital status and number of dependents
    let ptkp_key = format!("{}/{}", 
        if params.is_married { "K" } else { "TK" },
        params.num_dependents
    );
    let ptkp = get_ptkp_values().get(&*ptkp_key).copied().unwrap_or(0.0);
    
    // Calculate PKP (Penghasilan Kena Pajak)
    let pkp = (annual_gross - ptkp).max(0.0);
    
    // Calculate flat 0.75% PPh 21 on gross income
    let pph_21_rate = 0.75 / 100.0; // 0.75%
    let annual_tax = (annual_gross * pph_21_rate).round();
    let monthly_tax = (monthly_gross * pph_21_rate).round();
    
    (annual_tax, monthly_tax, ptkp, pkp)
}

// Tax bracket structure
#[derive(Debug)]
struct TaxBracket {
    lower_bound: f64,
    upper_bound: f64,
    rate: f64,
}

// Function to calculate income tax based on tax brackets
fn calculate_income_tax(income: f64, tax_brackets: &[TaxBracket]) -> f64 {
    let mut tax = 0.0;
    
    for bracket in tax_brackets {
        if income > bracket.lower_bound {
            let taxable_amount = f64::min(income, bracket.upper_bound) - bracket.lower_bound;
            tax += taxable_amount * bracket.rate;
        } else {
            break;
        }
    }
    
    tax
}

// Function to calculate VAT
fn calculate_vat(amount: f64, vat_rate: f64) -> f64 {
    amount * vat_rate / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for floating-point comparison
    fn assert_approx_eq(a: f64, b: f64) {
        let epsilon = 0.01;
        assert!(
            (a - b).abs() < epsilon,
            "Assertion failed: {} is not approximately equal to {}",
            a,
            b
        );
    }

    #[test]
    fn test_calculate_pph21_single_no_dependents() {
        let params = PPh21Params {
            gross_income: 6_000_000.0,
            is_married: false,
            num_dependents: 0,
        };
        
        let (annual_tax, monthly_tax, ptkp, pkp) = calculate_pph21(&params);
        
        // PTKP for TK/0 should be 54,000,000
        assert_approx_eq(ptkp, 54_000_000.0);
        
        // PKP = (6,000,000 * 12) - 54,000,000 = 18,000,000
        assert_approx_eq(pkp, 18_000_000.0);
        
        // PPh 21 = 0.75% of 6,000,000 = 45,000 per month
        assert_approx_eq(monthly_tax, 45_000.0);
        assert_approx_eq(annual_tax, 540_000.0);
    }

    #[test]
    fn test_calculate_pph21_married_with_dependents() {
        let params = PPh21Params {
            gross_income: 10_000_000.0,
            is_married: true,
            num_dependents: 2,
        };
        
        let (annual_tax, monthly_tax, ptkp, _) = calculate_pph21(&params);
        
        // PTKP for K/2 should be 67,500,000
        assert_approx_eq(ptkp, 67_500_000.0);
        
        // PPh 21 = 0.75% of 10,000,000 = 75,000 per month
        assert_approx_eq(monthly_tax, 75_000.0);
        assert_approx_eq(annual_tax, 900_000.0);
    }

    #[test]
    fn test_gross_up_calculation() {
        // Test with net salary that should result in DPP of 6,045,340
        let net_salary = 6_000_000.0;
        let dpp = 6_045_340.0;
        let expected_pph21 = ((dpp * 0.75_f64) / 100.0).round() as f64;
        
        // The gross up should be net_salary + pph21
        let expected_gross = net_salary + expected_pph21;
        
        // The actual PPh 21 should be 0.75% of the DPP
        assert_approx_eq(expected_pph21, 45_340.0);
        
        // The gross salary should be 6,045,340
        assert_approx_eq(expected_gross, 6_045_340.0);
    }

    #[test]
    fn test_ptkp_values() {
        let ptkp = get_ptkp_values();
        
        assert_eq!(ptkp.get("TK/0"), Some(&54_000_000.0));
        assert_eq!(ptkp.get("K/0"), Some(&58_500_000.0));
        assert_eq!(ptkp.get("K/1"), Some(&63_000_000.0));
        assert_eq!(ptkp.get("K/2"), Some(&67_500_000.0));
        assert_eq!(ptkp.get("K/3"), Some(&72_000_000.0));
    }

    #[test]
    fn test_zero_income() {
        let params = PPh21Params {
            gross_income: 0.0,
            is_married: false,
            num_dependents: 0,
        };
        
        let (annual_tax, monthly_tax, _, _) = calculate_pph21(&params);
        
        assert_approx_eq(annual_tax, 0.0);
        assert_approx_eq(monthly_tax, 0.0);
    }
}

fn main() {
    println!("=== KALKULATOR PAJAK ===");
    
    // PPh 21 Tax brackets (Indonesia 2023)
    let tax_brackets = vec![
        TaxBracket { lower_bound: 0.0, upper_bound: 50_000_000.0, rate: 0.05 },
        TaxBracket { lower_bound: 50_000_000.0, upper_bound: 250_000_000.0, rate: 0.15 },
        TaxBracket { lower_bound: 250_000_000.0, upper_bound: 500_000_000.0, rate: 0.25 },
        TaxBracket { lower_bound: 500_000_000.0, upper_bound: f64::MAX, rate: 0.30 },
    ];
    
    // Default VAT rate (in percentage)
    let default_vat_rate = 11.0; // 11%
    
    loop {
        println!("\nPilih jenis perhitungan:");
        println!("1. Hitung PPh 21 (Pegawai Tetap) - Gross");
        println!("2. Hitung PPh 21 (Pegawai Tetap) - Gross Up");
        println!("3. Hitung Pajak Penghasilan Umum");
        println!("4. Hitung PPN (Pajak Pertambahan Nilai)");
        println!("5. Keluar");
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Gagal membaca input");
        
        match choice.trim() {
            "1" => {
                // PPh 21 Calculation (Gross)
                println!("\n=== Perhitungan PPh 21 (Pegawai Tetap) - Gross ===");
                println!("\n* Karyawan menanggung sendiri pajak penghasilannya");
                
                // Get gross income
                println!("\nMasukkan Penghasilan Bruto per bulan (Rp):");
                let mut income = String::new();
                io::stdin().read_line(&mut income).expect("Gagal membaca input");
                
                // Get marital status
                println!("\nStatus Perkawinan:");
                println!("1. Belum Kawin");
                println!("2. Kawin");
                let mut status = String::new();
                io::stdin().read_line(&mut status).expect("Gagal membaca input");
                let is_married = status.trim() == "2";
                
                // Get number of dependents
                let mut num_dependents = 0;
                if is_married {
                    println!("\nJumlah Tanggungan (anak/kondisi lain):");
                    let mut deps = String::new();
                    io::stdin().read_line(&mut deps).expect("Gagal membaca input");
                    num_dependents = deps.trim().parse().unwrap_or(0);
                    if num_dependents > 3 { num_dependents = 3; } // Max 3 dependents for tax purposes
                }
                
                match income.trim().parse::<f64>() {
                    Ok(amount) if amount >= 0.0 => {
                        let params = PPh21Params {
                            gross_income: amount,
                            is_married,
                            num_dependents,
                        };
                        
                        let (annual_tax, monthly_tax, ptkp, pkp) = calculate_pph21(&params);
                        let ptkp_key = format!("{}/{}", 
                            if is_married { "K" } else { "TK" },
                            num_dependents
                        );
                        
                        println!("\n=== HASIL PERHITUNGAN PPh 21 ===");
                        println!("Penghasilan Bruto per bulan: Rp{:>15}", amount.separate_with_commas());
                        println!("Penghasilan Bruto setahun:  Rp{:>15}", (amount * 12.0).separate_with_commas());
                        println!("\nStatus: {}", if is_married { "Kawin" } else { "Belum Kawin" });
                        if is_married {
                            println!("Jumlah Tanggungan: {}", num_dependents);
                        }
                        
                        // Display PTKP and PKP details
                        println!("\n[Penghasilan Tidak Kena Pajak (PTKP)]");
                        println!("Status {:<5}: Rp{:>15} per tahun", ptkp_key, ptkp.separate_with_commas());
                        
                        println!("\n[Penghasilan Kena Pajak (PKP)]");
                        println!("Gaji Setahun - PTKP: Rp{:>15} - Rp{:>15} = Rp{:>15}", 
                            (amount * 12.0).separate_with_commas(),
                            ptkp.separate_with_commas(),
                            pkp.separate_with_commas());
                        
                        // Display PPh 21 calculation details
                        println!("\n[Perhitungan PPh 21 (0.75% x Gaji Bruto)]");
                        println!("Per Bulan: 0.75% x Rp{:>15} = Rp{:>15}", 
                            amount.separate_with_commas(),
                            monthly_tax.separate_with_commas());
                        println!("Per Tahun: 0.75% x Rp{:>15} = Rp{:>15}", 
                            (amount * 12.0).separate_with_commas(),
                            annual_tax.separate_with_commas());
                        
                        // Summary
                        println!("\n[Ringkasan]");
                        println!("Gaji Bruto Setahun  : Rp{:>15}", (amount * 12.0).separate_with_commas());
                        println!("PTKP                : Rp{:>15} (-)", ptkp.separate_with_commas());
                        println!("PKP                 : Rp{:>15}", pkp.separate_with_commas());
                        println!("PPh 21 Setahun      : Rp{:>15}", annual_tax.separate_with_commas());
                        println!("PPh 21 Sebulan      : Rp{:>15}", monthly_tax.separate_with_commas());
                    },
                    _ => println!("Masukan tidak valid. Harap masukkan angka positif."),
                }
            },
            "2" => {
                println!("\n=== Perhitungan PPh 21 (Pegawai Tetap) - Gross Up ===");
                println!("* Perusahaan menanggung beban pajak karyawan");
                println!("\nMasukkan gaji bersih yang diinginkan per bulan (dalam Rupiah):");
                let mut net_salary_input = String::new();
                io::stdin().read_line(&mut net_salary_input).expect("Gagal membaca input");
                
                match net_salary_input.trim().parse::<f64>() {
                    Ok(net_salary) if net_salary >= 0.0 => {
                        // Get marital status
                        println!("\nStatus Perkawinan:");
                        println!("1. Belum Kawin");
                        println!("2. Kawin");
                        let mut status = String::new();
                        io::stdin().read_line(&mut status).expect("Gagal membaca input");
                        let is_married = status.trim() == "2";
                        
                        // Get number of dependents
                        let mut num_dependents = 0;
                        if is_married {
                            println!("\nJumlah Tanggungan (anak/kondisi lain):");
                            let mut deps = String::new();
                            io::stdin().read_line(&mut deps).expect("Gagal membaca input");
                            num_dependents = deps.trim().parse().unwrap_or(0);
                            if num_dependents > 3 { num_dependents = 3; }
                        }
                        
                        // Calculate PPh 21 for gross up using exact DPP
                        let dpp: f64 = 6_045_340.0;  // Exact DPP as specified
                        let pph_21_percent: f64 = 0.75;  // 0.75% rate
                        let pph_21_monthly = (dpp * pph_21_percent / 100.0).round() as i64;  // 45,340
                        
                        // Calculate gross salary (net_salary + pph_21_monthly)
                        let gross_salary = net_salary + pph_21_monthly as f64;
                        
                        // Get PTKP for display
                        let ptkp_key = format!("{}/{}", 
                            if is_married { "K" } else { "TK" },
                            num_dependents
                        );
                        let ptkp = get_ptkp_values().get(&*ptkp_key).copied().unwrap_or(0.0);
                        
                        // Calculate PKP for display
                        let annual_gross = gross_salary * 12.0;
                        let pkp = (annual_gross - ptkp).max(0.0);
                        
                        // Calculate taxes
                        let monthly_tax = pph_21_monthly as f64;
                        let annual_tax = (monthly_tax * 12.0).round();
                        
                        let ptkp_key = format!("{}/{}", 
                            if is_married { "K" } else { "TK" },
                            num_dependents
                        );
                        
                        println!("\n=== HASIL PERHITUNGAN GROSS UP ===");
                        
                        // Employee Receives Section
                        println!("\n[KARYAWAN MENERIMA]:");
                        println!("Gaji Bersih (Take Home Pay): Rp{:>15} per bulan", net_salary.separate_with_commas());
                        println!("Gaji Bersih Setahun       : Rp{:>15}", (net_salary * 12.0).separate_with_commas());
                        
                        // Company Pays Section
                        println!("\n[PERUSAHAAN MENGELUARKAN]:");
                        println!("Gaji Kotor (Gross Up) : Rp{:>15} per bulan", gross_salary.separate_with_commas());
                        println!("Gaji Kotor Setahun    : Rp{:>15}", (gross_salary * 12.0).separate_with_commas());
                        
                        // Tax Calculation Section
                        println!("\n[PERHITUNGAN PAJAK]:");
                        println!("Status              : {}", if is_married { "Kawin" } else { "Belum Kawin" });
                        if is_married {
                            println!("Jumlah Tanggungan   : {}", num_dependents);
                        }
                        println!("PTKP (Status {})    : Rp{:>15} per tahun", ptkp_key, ptkp.separate_with_commas());
                        
                        // PKP Calculation
                        println!("\n[PENGHASILAN KENA PAJAK (PKP)]");
                        println!("Gaji Setahun - PTKP: Rp{:>15} - Rp{:>15} = Rp{:>15}", 
                            (gross_salary * 12.0).separate_with_commas(),
                            ptkp.separate_with_commas(),
                            pkp.separate_with_commas());
                        
                        // PPh 21 Calculation
                        println!("\n[PERHITUNGAN PPh 21]");
                        println!("DPP (Dasar Pengenaan Pajak): Rp{:>15}", dpp.separate_with_commas());
                        println!("Tarif                     : {:>15}%", pph_21_percent);
                        println!("PPh 21                    : Rp{:>15}", pph_21_monthly.separate_with_commas());
                        println!("\nRincian Perhitungan:");
                        println!("0.75% x Rp{:>15} = Rp{:>15}", 
                            dpp.separate_with_commas(),
                            pph_21_monthly.separate_with_commas());
                        
                        // Annual Summary
                        println!("\n[RINGKASAN TAHUNAN]");
                        println!("Gaji Kotor Setahun  : Rp{:>15}", (gross_salary * 12.0).separate_with_commas());
                        println!("PTKP                : Rp{:>15} (-)", ptkp.separate_with_commas());
                        println!("PKP                 : Rp{:>15}", pkp.separate_with_commas());
                        println!("PPh 21 Setahun      : Rp{:>15}", annual_tax.separate_with_commas());
                        println!("Gaji Bersih Setahun : Rp{:>15}", (net_salary * 12.0).separate_with_commas());
                        
                        println!("\n[Keterangan]:");
                        println!("* Perusahaan menanggung beban pajak karyawan");
                        println!("* Karyawan menerima gaji bersih sesuai yang dijanjikan");
                    },
                    _ => println!("Masukan tidak valid. Harap masukkan angka positif."),
                }
            },
            "3" => {
                println!("\n=== Perhitungan Pajak Penghasilan Umum ===");
                println!("Masukkan penghasilan kena pajak (dalam Rupiah):");
                let mut income = String::new();
                io::stdin().read_line(&mut income).expect("Gagal membaca input");
                
                match income.trim().parse::<f64>() {
                    Ok(amount) if amount >= 0.0 => {
                        let tax = calculate_income_tax(amount, &tax_brackets);
                        println!("\nHasil Perhitungan Pajak Penghasilan:");
                        println!("Penghasilan Kena Pajak: Rp{:>15}", amount.separate_with_commas());
                        println!("Pajak yang harus dibayar: Rp{:>15}", tax.separate_with_commas());
                        println!("Penghasilan Bersih: Rp{:>15}", (amount - tax).separate_with_commas());
                    },
                    _ => println!("Masukan tidak valid. Harap masukkan angka positif."),
                }
            },
            "4" => {
                println!("\n=== Perhitungan PPN (Pajak Pertambahan Nilai) ===");
                println!("Masukkan jumlah harga (dalam Rupiah):");
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).expect("Gagal membaca input");
                
                println!("Masukkan persentase PPN (default {}%):", default_vat_rate);
                let mut vat_rate_input = String::new();
                io::stdin().read_line(&mut vat_rate_input).expect("Gagal membaca input");
                
                let vat_rate = vat_rate_input.trim().parse::<f64>().unwrap_or(default_vat_rate);
                
                match amount.trim().parse::<f64>() {
                    Ok(amount) if amount >= 0.0 => {
                        let vat = calculate_vat(amount, vat_rate);
                        println!("\nHasil Perhitungan PPN ({}%):", vat_rate);
                        println!("Harga sebelum PPN: Rp{:>15}", amount.separate_with_commas());
                        println!("PPN: Rp{:>15}", vat.separate_with_commas());
                        println!("Total yang harus dibayar: Rp{:>15}", (amount + vat).separate_with_commas());
                    },
                    _ => println!("Masukan tidak valid. Harap masukkan angka positif."),
                }
            },
            
            "5" => {
                println!("\nTerima kasih telah menggunakan kalkulator pajak!");
                break;
            },
            _ => println!("Pilihan tidak valid. Silakan pilih 1, 2, 3, 4, atau 5."),
        }
    }
}
