//! Benchmarks for the micromail crate

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use micromail::{Config, Mail, Mailer};

fn bench_mail_format(c: &mut Criterion) {
    let config = Config::new("example.com");
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Test Body\nWith multiple lines\nAnd more lines\nAnd even more lines");
    
    c.bench_function("mail_format", |b| {
        b.iter(|| {
            black_box(mail.format(&config));
        });
    });
}

fn bench_mail_creation(c: &mut Criterion) {
    c.bench_function("mail_creation", |b| {
        b.iter(|| {
            let mail = Mail::new()
                .from("sender@example.com")
                .to("recipient@example.com")
                .subject("Test Subject")
                .body("Test Body");
            
            black_box(mail);
        });
    });
}

fn bench_config_creation(c: &mut Criterion) {
    c.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = Config::new("example.com")
                .timeout(std::time::Duration::from_secs(30))
                .use_tls(true)
                .ports(vec![25, 587, 465, 2525]);
            
            black_box(config);
        });
    });
}

fn bench_mailer_creation(c: &mut Criterion) {
    c.bench_function("mailer_creation", |b| {
        b.iter(|| {
            let config = Config::new("example.com");
            let mailer = Mailer::new(config);
            
            black_box(mailer);
        });
    });
}

criterion_group!(
    benches,
    bench_mail_format,
    bench_mail_creation,
    bench_config_creation,
    bench_mailer_creation,
);
criterion_main!(benches);