# Tailwind CSS Migration Guide

## Overview

This document describes the migration from Bootstrap to Tailwind CSS and the extraction of HTML templates into separate files.

## Changes Made

### 1. Template Extraction

HTML templates have been moved from inline strings in `src/templates.rs` to separate files in the `templates/` directory:

- `templates/index.html` - Main landing page
- `templates/no-vouchers.html` - No vouchers found page
- `templates/admin.html` - Admin panel
- `templates/network-vouchers.html` - Network vouchers listing
- `templates/voucher-card.html` - Individual voucher card template
- `templates/vouchers.html` - Voucher generation page template

### 2. CSS Framework Migration

**Before (Bootstrap 5.1.3):**
- Used Bootstrap classes like `container`, `row`, `col-*`, `card`, `btn`, etc.
- Required Bootstrap CSS and JS files
- Custom CSS for upload area and other components

**After (Tailwind CSS):**
- Modern utility-first CSS framework
- Responsive design with `sm:`, `md:`, `lg:` prefixes
- Consistent spacing and color system
- No custom CSS required

### 3. Key Design Improvements

#### Color Scheme
- Primary: Blue (`blue-600`, `blue-700`)
- Success: Green (`green-600`, `green-700`)
- Warning: Yellow (`yellow-500`, `yellow-600`)
- Error: Red (`red-600`, `red-700`)
- Gray scale for backgrounds and text

#### Components
- **Cards**: Clean white backgrounds with subtle shadows
- **Buttons**: Consistent padding, rounded corners, hover states
- **Forms**: Modern input styling with focus states
- **Tables**: Striped rows, hover effects, proper spacing
- **Badges**: Pill-shaped status indicators

#### Layout
- Container-based layout with proper responsive breakpoints
- Grid system using CSS Grid and Flexbox
- Consistent spacing using Tailwind's spacing scale

### 4. Template System Changes

**Before:**
```rust
pub const INDEX_TEMPLATE: &str = r#"<html>...</html>"#;
```

**After:**
```rust
fn load_template(name: &str) -> String {
    let template_path = format!("templates/{}.html", name);
    fs::read_to_string(&template_path).unwrap()
}

pub fn index_template() -> String {
    load_template("index")
}

pub fn generate_voucher_card(qr_code_base64: &str, network_name: &str, network_ssid: &str, voucher_code: &str) -> String {
    voucher_card_template()
        .replace("{{QR_CODE_BASE64}}", qr_code_base64)
        .replace("{{NETWORK_NAME}}", network_name)
        .replace("{{NETWORK_SSID}}", network_ssid)
        .replace("{{VOUCHER_CODE}}", voucher_code)
}
```

### 5. Dynamic Content Handling

Templates now use placeholder replacement for dynamic content:

- `{{NETWORK_OPTIONS}}` - Network dropdown options
- `{{NETWORK_ROWS}}` - Table rows for networks
- `{{VOUCHER_ROWS}}` - Table rows for vouchers
- `{{EMPTY_NETWORKS_MESSAGE}}` - Message when no networks exist
- `{{VOUCHER_COUNT}}` - Total voucher count
- `{{QR_CODE_BASE64}}` - Base64 encoded QR code image
- `{{NETWORK_SSID}}` - WiFi network SSID
- `{{VOUCHER_CODE}}` - Individual voucher code

### 6. Voucher Card Layout Enhancement

**New Layout Design:**
- **QR Code positioned on the right** - Easy to scan with mobile devices
- **Text information on the left** - Network name, voucher code, and instructions
- **Responsive flexbox layout** - Adapts to different screen sizes
- **Print-optimized** - Clean layout for physical voucher cards

**Template Structure:**
```html
<div class="flex items-center justify-between h-32">
    <!-- Left side - Text information -->
    <div class="flex-1 pr-4">
        <h3>WiFi Access Voucher</h3>
        <div>Network: {{NETWORK_SSID}}</div>
        <div>Voucher Code: {{VOUCHER_CODE}}</div>
    </div>

    <!-- Right side - QR Code -->
    <div class="flex-shrink-0">
        <img src="data:image/png;base64,{{QR_CODE_BASE64}}" />
    </div>
</div>
```

## Benefits

1. **Maintainability**: HTML templates are now separate files, easier to edit
2. **Modern Design**: Tailwind provides consistent, modern styling
3. **Performance**: Smaller CSS bundle (Tailwind purges unused styles)
4. **Responsiveness**: Better mobile experience with Tailwind's responsive utilities
5. **Consistency**: Unified design system across all pages
6. **Developer Experience**: Utility classes make styling faster and more predictable
7. **Template Modularity**: Voucher cards are now separate templates for easy customization
8. **Improved UX**: QR code positioning on the right makes scanning more intuitive
9. **Print Optimization**: Better layout for physical voucher printing

## Usage

The application now loads templates from the `templates/` directory at runtime. Ensure this directory exists and contains all required template files when running the application.

### Running the Application

```bash
cargo run -- --ssid "YourWiFiSSID" --password "YourWiFiPassword"
```

The server will start on `http://127.0.0.1:3000` and serve the new Tailwind-styled templates.

## File Structure

```
pfsense_portal_generator/
├── src/
│   ├── main.rs
│   ├── templates.rs          # Template loading logic
│   ├── voucher.rs
│   ├── wifi_network.rs
│   └── qr_generator.rs
├── templates/                # New template directory
│   ├── index.html            # Main landing page
│   ├── no-vouchers.html      # No vouchers found page
│   ├── admin.html            # Admin panel
│   ├── network-vouchers.html # Network vouchers listing
│   ├── voucher-card.html     # Individual voucher card template
│   └── vouchers.html         # Voucher generation page
└── ...
```

## Compatibility

This migration maintains full backward compatibility with existing functionality while providing a more modern and maintainable codebase.

## Voucher Card Customization

The voucher card design can now be easily customized by editing `templates/voucher-card.html`:

- **Layout**: Modify the flex layout to change QR code and text positioning
- **Styling**: Update Tailwind classes for colors, fonts, and spacing
- **Content**: Add additional fields like expiration dates or terms
- **Branding**: Include logos or custom styling elements

Example customization:
```html
<!-- Add a logo above the title -->
<div class="flex items-center mb-2">
    <img src="/logo.png" class="w-6 h-6 mr-2" alt="Logo">
    <h3 class="text-lg font-bold text-blue-800">Your Company WiFi</h3>
</div>
```
