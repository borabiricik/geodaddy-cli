# geodaddy — Test Guide (Phase 1)

## Ön Koşullar

- Rust toolchain kurulu olmalı (`rustup` ile). Kontrol: `rustc --version`
- `jq` kurulu olmalı (JSON çıktısını formatlamak için). macOS: `brew install jq`
- İnternet bağlantısı (testlerin bir kısmı `httpbin.org` kullanıyor)

---

## 1. Binary'i Build Et

```bash
cd cli
cargo build --release
```

İlk build ~1-2 dakika sürer (bağımlılıklar compile ediliyor). Başarılı olursa:

```
Finished `release` profile [optimized] target(s) in ...
```

Binary şuraya oluşur: `cli/target/release/geodaddy`

---

## 2. Temel Kullanım

### --help

```bash
./target/release/geodaddy --help
```

Beklenen çıktı:

```
GEO analysis tool — surface actionable AI search optimization issues

Usage: geodaddy [OPTIONS] <URL>

Arguments:
  <URL>  URL to analyze (supports http://localhost and http://127.0.0.1)

Options:
      --fail-under <SCORE>  Exit with code 1 if overall score is below this threshold (0-100)...
  -h, --help                Print help
  -V, --version             Print version
```

---

## 3. Bir URL Analiz Et

### Gerçek bir URL

```bash
./target/release/geodaddy https://example.com
```

Beklenen JSON çıktısı (stdout):

```json
{
  "schema_version": "1",
  "url": "https://example.com",
  "crawled_at": "2026-03-23T...",
  "pages": [
    {
      "url": "https://example.com/",
      "robots_blocked": false,
      "results": []
    }
  ]
}
```

> `results: []` boş çünkü analizörler Phase 2'de ekleniyor. Bu kasıtlı tasarım.

### jq ile formatlanmış çıktı

```bash
./target/release/geodaddy https://example.com | jq .
```

### Belirli bir alan

```bash
./target/release/geodaddy https://example.com | jq '.pages[0].robots_blocked'
```

---

## 4. Localhost Test

Çalışan bir yerel sunucu yokken bile hata vermeden çalışmalı:

```bash
./target/release/geodaddy http://localhost:3000
```

Beklenen: `robots_blocked: false`, exit code `0`

Eğer localhost'ta gerçek bir uygulama çalışıyorsa (örn. Next.js dev server):

```bash
./target/release/geodaddy http://localhost:3000/blog/post/1
```

---

## 5. robots.txt Davranışı

### robots.txt'i olan bir site

```bash
./target/release/geodaddy https://openai.com | jq '.pages[0].robots_blocked'
```

OpenAI'nin robots.txt'i `GPTBot` gibi botları engelliyor ama `geodaddy/0.1.0` user-agent'ını engellemiyorsa `false` döner.

### Kasıtlı engelleme testi

Eğer robots.txt'i `Disallow: /` olan bir site ile test etmek istersen `robots_blocked: true` görürsün — ama **crawl yine de devam eder** (soft warn davranışı).

---

## 6. --fail-under Flag'i

CI/CD pipeline entegrasyonu için:

```bash
# Phase 1'de score her zaman 0.0 (analizör yok) — threshold > 0 ise exit 1
./target/release/geodaddy --fail-under 50 https://example.com
echo "Exit code: $?"   # 1 beklenir

# threshold = 0 ise exit 0
./target/release/geodaddy --fail-under 0 https://example.com
echo "Exit code: $?"   # 0 beklenir
```

---

## 7. Otomatik Test Suite

Tüm 7 testi tek seferde çalıştır:

```bash
cd cli
chmod +x tests/integration_test.sh
bash tests/integration_test.sh
```

Beklenen çıktı:

```
PASS: --help exits 0 and contains docs
PASS: JSON structure: schema_version, pages[0] with url/robots_blocked/results
PASS: --fail-under 50 exits 1 (score=0.0 < 50)
PASS: --fail-under 0 exits 0 (score=0.0 >= 0)
PASS: stdout is valid JSON (no tracing noise)
PASS: localhost with no server: robots_blocked=false (graceful)

Results: 7 passed, 0 failed
```

> Testlerin 2-6'sı `http://httpbin.org/get` adresine istek atıyor. İnternet bağlantısı olmadan bu testler fail olabilir.

---

## 8. JSON stdout Temizliği (Pipe Testi)

JSON çıktısının tracing/log içermediğini doğrula:

```bash
./target/release/geodaddy https://example.com 2>/dev/null | jq .
```

`jq` hata vermeden parse ediyorsa stdout temiz demektir.

Verbose tracing görmek istersen:

```bash
RUST_LOG=debug ./target/release/geodaddy https://example.com 2>&1 | head -20
```

---

## 9. Hata Durumları

### Geçersiz URL

```bash
./target/release/geodaddy "not-a-url"
```

Beklenen: stderr'e hata mesajı, exit code `1`

### Ulaşılamayan host

```bash
./target/release/geodaddy https://this-domain-does-not-exist-xyz.com
```

Beklenen: JSON çıktısı üretilir, `robots_blocked: false`

---

## 10. CI/CD Entegrasyonu

```bash
# Başarı kriteri: siteye ulaşılabilir + threshold 0
./target/release/geodaddy --fail-under 0 https://example.com
# Phase 2 sonrası kullanım:
# ./target/release/geodaddy --fail-under 70 https://mysite.com
```

---

## Özet

| Test | Komut | Beklenen |
|------|-------|----------|
| Build | `cargo build --release` | Compile başarılı |
| Yardım | `geodaddy --help` | Kullanım belgesi |
| Temel analiz | `geodaddy https://example.com` | JSON çıktısı |
| Localhost | `geodaddy http://localhost:3000` | Exit 0, robots_blocked false |
| Exit code fail | `geodaddy --fail-under 50 <url>` | Exit 1 |
| Exit code pass | `geodaddy --fail-under 0 <url>` | Exit 0 |
| Otomatik testler | `bash tests/integration_test.sh` | 7/7 PASS |
