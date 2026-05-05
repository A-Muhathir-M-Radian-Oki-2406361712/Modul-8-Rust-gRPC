# Reflection: gRPC & Rust Implementation

## 1. Perbedaan Unary, Server Streaming, dan Bi-directional Streaming
* **Unary:** 1 Request menghasilkan 1 Response. Cocok untuk operasi API standar seperti CRUD atau pemrosesan pembayaran.
* **Server Streaming:** 1 Request menghasilkan banyak Response (aliran data). Cocok untuk fitur *live feed*, unduhan file bertahap, atau notifikasi sistem.
* **Bi-directional:** Klien dan server dapat mengirim banyak pesan secara simultan dalam satu koneksi. Cocok untuk aplikasi *chat real-time* atau *game multiplayer*.

## 2. Keamanan gRPC di Rust
* **Enkripsi:** Wajib menggunakan TLS (Transport Layer Security) via `tonic::transport::ServerTlsConfig` untuk mengenkripsi data yang transit.
* **Autentikasi:** Menerapkan *interceptor* di sisi server untuk memvalidasi token (misal: JWT) dari metadata/header gRPC.
* **Otorisasi:** Mengecek hak akses pengguna (Role-Based Access Control) di dalam *handler* fungsi setelah autentikasi berhasil.

## 3. Tantangan Bi-directional Streaming di Rust (Aplikasi Chat)
* **Manajemen State:** Mengelola status koneksi klien secara konkuren (misal menggunakan `Arc<Mutex<...>>` atau *actor model*) tanpa memicu *deadlock*.
* **Penanganan Koneksi Terputus:** Mengelola memori dan saluran data (*channel*) agar tidak terjadi kebocoran memori (*memory leak*) atau *broken pipe* saat klien tiba-tiba *disconnect*.

## 4. `ReceiverStream` di `tokio_stream`
* **Kelebihan:** Sangat mudah diintegrasikan dengan ekosistem `tokio` karena langsung mengubah *channel* asinkron `mpsc` menjadi sebuah *Stream* standar yang dibutuhkan `tonic`.
* **Kekurangan:** Menambah sedikit *overhead* memori karena bergantung pada *buffer channel*. Jika klien lambat membaca data, server harus menangani isu *backpressure* agar *buffer* tidak penuh.

## 5. Struktur Kode Rust untuk Skalabilitas
* **Pemisahan Layer:** Pisahkan definisi Protobuf (`build.rs`), *domain logic/services*, *network handlers* (implementasi `tonic`), dan akses data (*repository*).
* **Penggunaan Trait:** Gunakan `trait` untuk interaksi antar modul agar kode mudah diuji (*mocking*) dan fleksibel saat ada perubahan implementasi di masa depan.

## 6. Pengembangan Kompleksitas `MyPaymentService`
* **Integrasi Gateway:** Menghubungkan servis dengan API *payment gateway* pihak ketiga (Midtrans, Stripe, dll).
* **Idempotensi & Transaksi ACID:** Memastikan satu *request* tidak ditagih dua kali (*double charge*) dan menggunakan transaksi *database* yang aman jika proses gagal di tengah jalan.
* **Audit & Logging:** Mencatat setiap langkah transaksi untuk keperluan rekonsiliasi dan investigasi.

## 7. Dampak gRPC pada Arsitektur Sistem Terdistribusi
gRPC mendorong arsitektur *microservices* yang sangat cepat dan mengikat (*strongly-typed contract*). Namun, sistem memerlukan penyesuaian interoperabilitas, seperti menggunakan **gRPC-Web** atau **API Gateway** (Penerjemah REST ke gRPC) karena *browser* web biasa belum mendukung protokol HTTP/2 secara mentah.

## 8. HTTP/2 (gRPC) vs HTTP/1.1 (REST/WebSocket)
* **Keunggulan HTTP/2:** Mendukung *multiplexing* (banyak request dalam 1 koneksi), kompresi header (HPACK), dan format biner yang jauh lebih ringan dan cepat daripada teks biasa.
* **Kelemahan HTTP/2:** Format binernya tidak bisa dibaca langsung oleh manusia sehingga lebih sulit untuk di-*debug* (butuh alat khusus), serta tidak didukung oleh perangkat/klien versi lama.

## 9. Model Request-Response (REST) vs Bi-directional (gRPC)
REST API bergantung pada *polling* (klien terus bertanya ke server apakah ada data baru) yang boros *bandwidth* dan memiliki latensi lebih tinggi. Bi-directional gRPC menjaga koneksi tetap terbuka, sehingga server bisa melakukan *push* data secara *real-time* dengan latensi sangat rendah.

## 10. Pendekatan Skema (Protobuf) vs Tanpa Skema (JSON)
* **Implikasi Protobuf:** Payload berukuran lebih kecil dan tipe data terjamin ketat (*type-safe*). Perubahan struktur harus dikelola dengan hati-hati agar tidak merusak versi sebelumnya (*backward compatibility*).
* **Implikasi JSON:** Jauh lebih fleksibel dan mudah dibaca manusia tanpa perlu proses kompilasi kode, namun ukurannya lebih besar, lebih lambat diurai (*parsing*), dan rawan kesalahan tipe data saat *runtime*.