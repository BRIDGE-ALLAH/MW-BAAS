# MW-BAAS

### Infrastruktur di Balik Setiap Kesaksian.
**Platform backend modular untuk ekosistem MoonWitness.**

*Satu Fondasi. Banyak Aplikasi. Dapat Dipertanggungjawabkan Sejak Desain.*

MW-BAAS adalah platform backend yang dirancang untuk mendukung produk-produk dalam ekosistem MoonWitness. MW-BAAS menyediakan layanan untuk aplikasi—HTTP API, identitas dan akses, pekerjaan latar belakang, penyimpanan, cache, notifikasi, integrasi, dan operasional—dengan **MoonWitness ORM (MW-ORM)** sebagai fondasi relasional dan data modular yang dapat digunakan kembali.

MW-BAAS memungkinkan berbagai aplikasi berbagi infrastruktur andal tanpa harus meleburkan logika produk masing-masing.

> The Moon Watches. The Record Remembers. The Evidence Remains Inspectable.

Prinsip ini berangkat dari visi MoonWitness: menjaga materi asli, menelusuri klaim ke sumbernya, membedakan bukti dari interpretasi, menghormati persetujuan dan hak, serta memastikan keputusan penting dapat dipertanggungjawabkan oleh manusia.

## Identitas produk

| Lapisan | Tanggung jawab |
|---|---|
| **Empire of the Light** | Visi induk dan konteks ekosistem |
| **MoonWitness** | Ekosistem pengetahuan, kesaksian, riset, dan storytelling untuk publik |
| **MW-BAAS** | Platform backend bersama dan layanan aplikasi |
| **MW-ORM** | Mesin relasional dan fondasi data modular |
| **Applications** | Produk independen di atas kontrak bersama |

Setiap aplikasi tetap bertanggung jawab atas pengalaman pengguna, kebijakan, dan keputusan domainnya.

## Arsitektur dan batas tanggung jawab

```text
Aplikasi MoonWitness
         |
         v
      MW-BAAS
 API · Identity · Access
 Jobs · Storage · Cache
 Notifications · Operations
         |
         v
 Integrasi MW-ORM
 Relational core · Entities
 Relations · Queries · Transactions
 Kontrak modul (sesuai implementasi)
         |
         v
 PostgreSQL · MySQL · SQLite
```

**Arah dependensi:** MW-BAAS menggunakan MW-ORM. MW-ORM harus tetap dapat digunakan mandiri dan tidak boleh bergantung pada logika produk BAAS.

- **MW-ORM memiliki** primitif relasional, metadata entitas/relasi, persistence, transaksi, dan kontrak modul/add-on generik sesuai implementasi.
- **MW-BAAS memiliki** kapabilitas backend, komposisi API, kebijakan identitas dan otorisasi, alur kerja tenant, integrasi, dan deployment.
- **Aplikasi memiliki** aturan domain dan kebijakan data spesifik produk.

Setiap schema, riwayat migrasi, dan batas transaksi harus memiliki satu pemilik otoritatif. Jangan menjalankan dua ORM sebagai pemilik yang saling bersaing atas keadaan database yang sama.

## Hubungan dengan MW-ORM

MW-ORM berkembang secara independen dari codebase SeaORM 2.0. MW-BAAS mematok dependensi `sea-orm` ke revisi Git MW-ORM; patch Cargo juga memastikan `sea-orm-migration` memakai sumber ORM yang sama. Feature opsional `mw-modules` mengekspos kernel eksperimental `moonwitness-module` melalui `loco_rs::modules`. Manifest, resolver, registry, dan API lifecycle sinkron tersedia pada revisi tersebut, tetapi koordinasi migrasi database dan perilaku add-on runtime belum diimplementasikan.

Strategi integrasi MW-BAAS dilakukan bertahap:

1. Pertahankan adapter database yang berjalan sambil mengaudit kompatibilitas.
2. Tetapkan batas integrasi yang sempit dan berversi; jangan masukkan logika BAAS ke ORM core.
3. Integrasikan MW-ORM melalui adapter eksplisit dan tes kompatibilitas.
4. Pindahkan kepemilikan model dan migrasi setelah pembuktian end-to-end.
5. Hapus jalur persistence duplikat setelah aplikasi konsumen lulus gerbang migrasi dan kompatibilitas.

Baca [Kontrak Integrasi MW-ORM](docs/architecture/MW-ORM-INTEGRATION.md) dan [Narasi Branding MW-BAAS](docs/brand/BRAND.md).

## Status implementasi saat ini

Repositori ini saat ini berisi codebase framework Rust turunan Loco. Metadata paket dan integrasi database masih mengidentifikasi Loco dan SeaORM. Identitas MW-BAAS dan integrasi MW-ORM merupakan arah pengembangan—bukan bukti bahwa penggantian nama paket atau adapter MW-ORM siap produksi telah selesai.

| Area | Status |
|---|---|
| Narasi produk dan batas tanggung jawab | Terdokumentasi |
| Pin sumber MW-ORM Core | Dikonfigurasi; menunggu validasi CI |
| Fondasi HTTP/backend saat ini | Ada; perlu audit khusus MW-BAAS |
| Migrasi identitas paket/crate | Belum dimulai; sensitif terhadap kompatibilitas |
| Kernel modul MW-ORM | API opsional tersedia; tes integrasi dan CI menunggu |
| Kesiapan produksi | Tidak diklaim |

## Prinsip desain

1. **Satu otoritas data** — satu pemilik kanonis untuk schema, migrasi, dan transaksi.
2. **Kontrak eksplisit** — API berversi antara ORM, backend, dan aplikasi.
3. **Modular melalui komposisi** — kapabilitas opsional tidak menjadi dependensi wajib.
4. **Akuntabilitas manusia** — jaga provenance dan auditability untuk catatan atau riset yang memerlukannya.
5. **Privasi dan persetujuan** — kumpulkan dan tampilkan hanya data yang berhak ditangani.
6. **Aman sejak awal** — default deny, validasi input, isolasi tenant, dan tes jalur bypass.
7. **Operasional teramati** — log, metrik, trace, health check, dan diagnostik.
8. **Klaim yang jujur** — bedakan implementasi dari proposal dan roadmap.

## Roadmap dan gerbang penerimaan

- **Fondasi:** audit repositori, lisensi, dependensi, dan CI.
- **Identitas:** konsistensi nama MW-BAAS, dokumentasi, dan rencana migrasi paket.
- **Jembatan ORM:** revisi MW-ORM yang dipatok, adapter, matriks fitur, dan tes kompatibilitas.
- **Modul layanan:** API, identity/access, storage, jobs, events, notifications, kebijakan tenant.
- **Aplikasi referensi:** demonstrasi aplikasi nyata yang menggunakan MW-BAAS dan MW-ORM bersama.
- **Kesiapan rilis:** tinjauan keamanan, simulasi migrasi, dokumentasi operasional, dan CI reproducible.

Milestone hanya selesai jika tes penerimaan dan bukti tersedia. README atau diagram saja bukan bukti implementasi.

## Pengembangan

Repositori ini sedang bertransisi dari fondasi turunan Loco menuju MW-BAAS. Sampai migrasi paket selesai, gunakan workspace Rust dan panduan pengembangan yang ada sebelum mengubah internal framework. Jangan mengganti nama crate, dependensi ORM, atau kepemilikan migrasi hanya sebagai perubahan branding.

## Referensi

- [Visi MoonWitness XV](https://github.com/bjo163/moonwitnessxv/blob/main/docs/vision/MASTER_VISION_NARRATIVE_XV.md)
- [Repositori MW-ORM](https://github.com/BRIDGE-ALLAH/MW-ORM)
- [Arsitektur MW-ORM](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/ARCHITECTURE.md)
- [Blueprint implementasi MW-ORM](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/BLUEPRINT.md)
- [Kontrak integrasi](docs/architecture/MW-ORM-INTEGRATION.md)
- [Narasi branding](docs/brand/BRAND.md)

---
**MW-BAAS — One Foundation. Many Applications. Accountable by Design.**
