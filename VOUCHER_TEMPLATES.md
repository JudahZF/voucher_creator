# Voucher Template Customization Guide

## Overview

The pfSense Portal Generator now uses modular HTML templates for voucher cards, making it easy to customize the appearance and layout of generated vouchers without modifying the Rust code.

## Template Files

### `templates/voucher-card.html`
This is the template for individual voucher cards. Each voucher in your CSV file will generate one card using this template.

### `templates/vouchers.html`
This is the main page template that contains the header, navigation, and grid layout for displaying multiple voucher cards.

## Current Layout

The default voucher card layout features:
- **QR code on the right** - 96x96 pixel QR code for easy mobile scanning
- **Text information on the left** - Network name, voucher code, and instructions
- **Horizontal layout** - Optimized for both screen viewing and printing
- **Tailwind CSS styling** - Modern, responsive design

## Customization Examples

### 1. Change QR Code Position to Top

To move the QR code above the text content, modify `voucher-card.html`:

```html
<div class="bg-white border-2 border-blue-500 rounded-lg shadow-md p-4 mb-4 voucher-card" style="page-break-inside: avoid;">
    <!-- Top - QR Code -->
    <div class="flex justify-center mb-4">
        <div class="w-24 h-24 border border-gray-300 rounded p-1 bg-white">
            <img src="data:image/png;base64,{{QR_CODE_BASE64}}" 
                 alt="WiFi QR Code" 
                 class="w-full h-full object-contain">
        </div>
    </div>
    
    <!-- Bottom - Text information -->
    <div class="text-center">
        <h3 class="text-lg font-bold text-gray-800 mb-2">WiFi Access Voucher</h3>
        <div class="space-y-1">
            <div class="text-sm">
                <span class="font-medium text-gray-700">Network:</span>
                <span class="text-gray-600">{{NETWORK_SSID}}</span>
            </div>
            <div class="bg-gray-100 px-2 py-1 rounded text-xs font-mono font-bold text-gray-800 break-all">
                {{VOUCHER_CODE}}
            </div>
            <div class="text-xs text-gray-500 mt-1">
                Scan QR code to connect
            </div>
        </div>
    </div>
</div>
```

### 2. Add Company Branding

Add your company logo and colors:

```html
<div class="bg-white border-2 border-purple-500 rounded-lg shadow-md p-4 mb-4 voucher-card" style="page-break-inside: avoid;">
    <div class="flex items-center justify-between h-32">
        <!-- Left side - Text information with branding -->
        <div class="flex-1 pr-4">
            <div class="flex items-center mb-2">
                <img src="/assets/logo.png" class="w-8 h-8 mr-2" alt="Company Logo">
                <h3 class="text-lg font-bold text-purple-800">Company WiFi Access</h3>
            </div>
            <div class="space-y-1">
                <div class="text-sm">
                    <span class="font-medium text-gray-700">Network:</span>
                    <span class="text-purple-600 font-semibold">{{NETWORK_SSID}}</span>
                </div>
                <div class="text-sm">
                    <span class="font-medium text-gray-700">Access Code:</span>
                </div>
                <div class="bg-purple-100 px-2 py-1 rounded text-xs font-mono font-bold text-purple-800 break-all">
                    {{VOUCHER_CODE}}
                </div>
                <div class="text-xs text-purple-600 mt-1">
                    Scan QR • Valid 24 hours
                </div>
            </div>
        </div>
        
        <!-- Right side - QR Code with branded border -->
        <div class="flex-shrink-0">
            <div class="w-24 h-24 border-2 border-purple-300 rounded-lg p-1 bg-purple-50">
                <img src="data:image/png;base64,{{QR_CODE_BASE64}}" 
                     alt="WiFi QR Code" 
                     class="w-full h-full object-contain">
            </div>
        </div>
    </div>
</div>
```

### 3. Minimal Design

For a clean, minimal appearance:

```html
<div class="bg-white border border-gray-200 rounded p-3 mb-3 voucher-card" style="page-break-inside: avoid;">
    <div class="flex items-center justify-between">
        <div class="flex-1 pr-3">
            <div class="text-sm font-medium text-gray-900 mb-1">{{NETWORK_SSID}}</div>
            <div class="font-mono text-xs bg-gray-50 px-2 py-1 rounded">{{VOUCHER_CODE}}</div>
        </div>
        <div class="w-16 h-16">
            <img src="data:image/png;base64,{{QR_CODE_BASE64}}" 
                 alt="QR Code" 
                 class="w-full h-full">
        </div>
    </div>
</div>
```

### 4. Add Expiration Date

To include expiration information (requires modifying the Rust code to pass the date):

```html
<div class="space-y-1">
    <div class="text-sm">
        <span class="font-medium text-gray-700">Network:</span>
        <span class="text-gray-600">{{NETWORK_SSID}}</span>
    </div>
    <div class="text-sm">
        <span class="font-medium text-gray-700">Expires:</span>
        <span class="text-red-600">{{EXPIRATION_DATE}}</span>
    </div>
    <div class="bg-gray-100 px-2 py-1 rounded text-xs font-mono font-bold text-gray-800 break-all">
        {{VOUCHER_CODE}}
    </div>
</div>
```

## Page Template Customization

### Modify Grid Layout

Change the number of vouchers per row by editing `vouchers.html`:

```html
<!-- 2 columns on medium screens, 3 on large -->
<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 voucher-grid">

<!-- 1 column on all screens for large vouchers -->
<div class="grid grid-cols-1 gap-4 voucher-grid">

<!-- 3 columns on medium, 4 on large, 6 on extra-large -->
<div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4 voucher-grid">
```

### Add Custom Header

Modify the header section in `vouchers.html`:

```html
<div class="no-print bg-gradient-to-r from-blue-600 to-purple-600 text-white">
    <div class="container mx-auto px-6 py-8">
        <div class="text-center">
            <h1 class="text-4xl font-bold mb-2">
                <i class="fas fa-wifi mr-3"></i>Guest WiFi Vouchers
            </h1>
            <p class="text-xl text-blue-100">
                {{VOUCHER_COUNT}} access codes for {{NETWORK_NAME}}
            </p>
        </div>
    </div>
</div>
```

## Available Placeholders

When customizing templates, you can use these placeholders:

### Voucher Card Template (`voucher-card.html`)
- `{{QR_CODE_BASE64}}` - Base64 encoded QR code image
- `{{NETWORK_SSID}}` - WiFi network name
- `{{VOUCHER_CODE}}` - Individual voucher access code

### Vouchers Page Template (`vouchers.html`)
- `{{VOUCHER_COUNT}}` - Total number of vouchers
- `{{NETWORK_NAME}}` - Display name of the network
- `{{NETWORK_SSID}}` - WiFi network SSID
- `{{VOUCHER_CARDS}}` - Generated voucher cards HTML

## Print Optimization

### Print-Specific Styling

Use the `.no-print` class for elements that shouldn't appear when printing:

```html
<div class="no-print">
    <button onclick="window.print()">Print Vouchers</button>
</div>
```

### Print Media Queries

The templates include print-optimized CSS:

```css
@media print {
    .no-print {
        display: none !important;
    }
    .voucher-card {
        break-inside: avoid;
        margin-bottom: 0.5rem;
    }
}
```

## Color Schemes

### Default Blue Theme
- Primary: `blue-500`, `blue-600`, `blue-700`
- Text: `gray-700`, `gray-800`, `gray-900`
- Background: `gray-50`, `gray-100`

### Alternative Themes

**Green Theme:**
```css
border-green-500 text-green-800 bg-green-50
```

**Purple Theme:**
```css
border-purple-500 text-purple-800 bg-purple-50
```

**Red Theme:**
```css
border-red-500 text-red-800 bg-red-50
```

## Tips for Customization

1. **Test Print Layout** - Always test your changes with the print preview
2. **Mobile Responsive** - Use Tailwind's responsive prefixes (`sm:`, `md:`, `lg:`)
3. **QR Code Size** - Keep QR codes at least 64x64 pixels for reliable scanning
4. **Font Legibility** - Use high contrast colors for voucher codes
5. **Break Points** - Include `page-break-inside: avoid` for proper printing

## Troubleshooting

### QR Code Not Displaying
- Ensure the `{{QR_CODE_BASE64}}` placeholder is present
- Check that the `src` attribute is correctly formatted

### Layout Issues
- Verify Tailwind CSS classes are spelled correctly
- Test responsive behavior at different screen sizes
- Use browser developer tools to debug CSS

### Print Problems
- Check print media queries are working
- Ensure `.no-print` class is applied to screen-only elements
- Test with different browsers and print settings

## Advanced Customization

For advanced changes requiring additional data, you may need to modify the Rust code in `src/templates.rs` to pass additional variables to the templates.

Example: Adding a timestamp requires updating the `generate_voucher_card` function to include a `{{TIMESTAMP}}` placeholder.