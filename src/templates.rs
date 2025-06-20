use crate::database::VoucherCounts;
use crate::wifi_network::WiFiNetwork;
use crate::voucher::Voucher;
use std::fs;

// Load template files at compile time or runtime
fn load_template(name: &str) -> String {
    let template_path = format!("templates/{}.html", name);
    fs::read_to_string(&template_path)
        .unwrap_or_else(|_| panic!("Failed to load template: {}", template_path))
}

pub fn no_vouchers_template() -> String {
    load_template("no-vouchers")
}

pub fn voucher_card_template() -> String {
    load_template("voucher-card")
}

pub fn vouchers_template() -> String {
    load_template("vouchers")
}

pub fn generate_voucher_card(
    qr_code_base64: &str,
    network_ssid: &str,
    network_name: &str,
    voucher_code: &str,
) -> String {
    voucher_card_template()
        .replace("{{QR_CODE_BASE64}}", qr_code_base64)
        .replace("{{NETWORK_SSID}}", network_ssid)
        .replace("{{NETWORK_NAME}}", network_name)
        .replace("{{VOUCHER_CODE}}", voucher_code)
}

pub fn generate_vouchers_page(
    voucher_count: usize,
    network_name: &str,
    network_ssid: &str,
    voucher_cards: &str,
) -> String {
    vouchers_template()
        .replace("{{VOUCHER_COUNT}}", &voucher_count.to_string())
        .replace("{{NETWORK_NAME}}", network_name)
        .replace("{{NETWORK_SSID}}", network_ssid)
        .replace("{{VOUCHER_CARDS}}", voucher_cards)
}

pub fn response_template() -> String {
    load_template("response")
}

pub fn success_response(title: &str, message: &str, voucher_count: usize, buttons: &str) -> String {
    let stats_section = if voucher_count > 0 {
        format!(
            r#"
            <div class="mt-6 bg-gradient-to-r from-green-50 to-emerald-50 rounded-2xl p-6 border border-green-200">
                <div class="text-center">
                    <div class="flex items-center justify-center space-x-6">
                        <div class="text-center">
                            <div class="text-3xl font-bold text-green-800">{}</div>
                            <div class="text-sm text-green-600 font-medium">Vouchers Loaded</div>
                        </div>
                        <div class="w-px h-12 bg-green-300"></div>
                        <div class="text-center">
                            <div class="text-3xl font-bold text-green-800">✓</div>
                            <div class="text-sm text-green-600 font-medium">Ready to Print</div>
                        </div>
                    </div>
                </div>
            </div>
            "#,
            voucher_count
        )
    } else {
        "".to_string()
    };

    response_template()
        .replace("{{TITLE}}", title)
        .replace("{{SUBTITLE}}", "Operation completed successfully")
        .replace(
            "{{HEADER_GRADIENT}}",
            "bg-gradient-to-r from-emerald-500 to-teal-600",
        )
        .replace("{{ICON_CLASS}}", "fas fa-check-circle")
        .replace("{{TEXT_COLOR_CLASS}}", "text-emerald-100")
        .replace(
            "{{MESSAGE_BG_CLASS}}",
            "bg-gradient-to-r from-green-50 to-emerald-50",
        )
        .replace("{{MESSAGE_BORDER_CLASS}}", "border-green-200")
        .replace(
            "{{MESSAGE_ICON_CLASS}}",
            "fas fa-check-circle text-green-600",
        )
        .replace("{{MESSAGE_TEXT_CLASS}}", "text-green-800")
        .replace("{{MESSAGE_TITLE}}", "Success!")
        .replace("{{MESSAGE_CONTENT}}", message)
        .replace("{{PRIMARY_BUTTON}}", buttons)
        .replace("{{SECONDARY_BUTTONS}}", "")
        .replace("{{ADDITIONAL_INFO}}", "")
        .replace("{{STATS_SECTION}}", &stats_section)
        .replace("{{AUTO_REDIRECT}}", "false")
        .replace("{{REDIRECT_URL}}", "")
        .replace("{{REDIRECT_DELAY}}", "0")
}

pub fn error_response(title: &str, message: &str, buttons: &str) -> String {
    response_template()
        .replace("{{TITLE}}", title)
        .replace(
            "{{SUBTITLE}}",
            "An error occurred while processing your request",
        )
        .replace(
            "{{HEADER_GRADIENT}}",
            "bg-gradient-to-r from-red-500 to-pink-600",
        )
        .replace("{{ICON_CLASS}}", "fas fa-exclamation-triangle")
        .replace("{{TEXT_COLOR_CLASS}}", "text-red-100")
        .replace(
            "{{MESSAGE_BG_CLASS}}",
            "bg-gradient-to-r from-red-50 to-pink-50",
        )
        .replace("{{MESSAGE_BORDER_CLASS}}", "border-red-200")
        .replace(
            "{{MESSAGE_ICON_CLASS}}",
            "fas fa-exclamation-circle text-red-600",
        )
        .replace("{{MESSAGE_TEXT_CLASS}}", "text-red-800")
        .replace("{{MESSAGE_TITLE}}", "Error!")
        .replace("{{MESSAGE_CONTENT}}", message)
        .replace("{{PRIMARY_BUTTON}}", buttons)
        .replace("{{SECONDARY_BUTTONS}}", "")
        .replace("{{ADDITIONAL_INFO}}", "")
        .replace("{{STATS_SECTION}}", "")
        .replace("{{AUTO_REDIRECT}}", "false")
        .replace("{{REDIRECT_URL}}", "")
        .replace("{{REDIRECT_DELAY}}", "0")
}

pub fn admin_template(networks: &[WiFiNetwork], voucher_counts: &[VoucherCounts]) -> String {
    let template = load_template("admin");

    let network_rows = networks
        .iter()
        .zip(voucher_counts.iter())
        .map(|(network, counts)| {
            let voucher_count = counts.total;
            let unused_count = counts.unused;
            let status_badge = if network.is_active {
                r#"<span class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-semibold bg-gradient-to-r from-green-100 to-emerald-100 text-green-800 border border-green-200">
                    <i class="fas fa-check-circle mr-1"></i>Active
                </span>"#
            } else {
                r#"<span class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-semibold bg-gradient-to-r from-red-100 to-pink-100 text-red-800 border border-red-200">
                    <i class="fas fa-times-circle mr-1"></i>Inactive
                </span>"#
            };

            format!(
                r#"
                <tr class="group hover:bg-gradient-to-r hover:from-blue-50 hover:to-indigo-50 transition-all duration-300 border-b border-gray-100">
                    <td class="px-6 py-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0 w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-lg flex items-center justify-center mr-4">
                                <i class="fas fa-wifi text-white text-sm"></i>
                            </div>
                            <div>
                                <div class="text-sm font-bold text-gray-900">{}</div>
                                <div class="text-xs text-gray-500 mt-1">Network</div>
                            </div>
                        </div>
                    </td>
                    <td class="px-6 py-6">
                        <div class="text-sm font-mono bg-gradient-to-r from-gray-100 to-gray-200 text-gray-800 px-3 py-2 rounded-lg border border-gray-300">
                            <i class="fas fa-broadcast-tower mr-2 text-gray-600"></i>{}
                        </div>
                    </td>
                    <td class="px-6 py-6">
                        {}
                    </td>
                    <td class="px-6 py-6">
                        <div class="text-sm text-gray-700 max-w-xs">
                            <i class="fas fa-info-circle mr-2 text-gray-400"></i>
                            {}
                        </div>
                    </td>
                    <td class="px-6 py-6">
                        <div class="flex items-center space-x-2">
                            <div class="bg-gradient-to-r from-emerald-100 to-teal-100 text-emerald-800 px-3 py-1 rounded-lg text-sm font-semibold border border-emerald-200">
                                <i class="fas fa-ticket-alt mr-1"></i>{} available
                            </div>
                            <div class="text-gray-500 text-sm">/ {} total</div>
                        </div>
                    </td>
                    <td class="px-6 py-6">
                        <div class="flex items-center space-x-2">
                            <a href="/admin/networks/{}/vouchers" class="bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-3 py-2 rounded-lg text-xs font-semibold transition-all duration-200 transform hover:scale-105 shadow-md hover:shadow-lg">
                                <i class="fas fa-list mr-1"></i>View
                            </a>
                            <a href="/generate?network_id={}" class="bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-600 hover:to-teal-700 text-white px-3 py-2 rounded-lg text-xs font-semibold transition-all duration-200 transform hover:scale-105 shadow-md hover:shadow-lg">
                                <i class="fas fa-print mr-1"></i>Generate
                            </a>
                            <form method="post" action="/admin/networks/{}/delete" class="inline">
                                <button type="submit" class="bg-gradient-to-r from-red-500 to-pink-600 hover:from-red-600 hover:to-pink-700 text-white px-3 py-2 rounded-lg text-xs font-semibold transition-all duration-200 transform hover:scale-105 shadow-md hover:shadow-lg" onclick="return confirm('⚠️ Are you sure you want to delete this network? This will also remove all associated vouchers.')">
                                    <i class="fas fa-trash mr-1"></i>Delete
                                </button>
                            </form>
                        </div>
                    </td>
                </tr>
                "#,
                network.name,
                network.ssid,
                status_badge,
                network.description.as_ref().unwrap_or(&"No description provided".to_string()),
                unused_count,
                voucher_count,
                network.id,
                network.id,
                network.id
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let network_options = networks
        .iter()
        .map(|network| {
            format!(
                r#"<option value="{}">{} ({})</option>"#,
                network.id, network.name, network.ssid
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let empty_networks_message = if networks.is_empty() {
        r#"<div class="text-center py-16">
            <div class="bg-gradient-to-br from-blue-50 to-indigo-100 rounded-2xl p-12 border border-blue-200">
                <i class="fas fa-network-wired text-6xl text-blue-400 mb-6"></i>
                <h3 class="text-2xl font-bold text-gray-800 mb-4">No Networks Yet</h3>
                <p class="text-gray-600 mb-6 max-w-md mx-auto">Get started by creating your first WiFi network using the form above. Once created, you can upload voucher codes and generate printable vouchers.</p>
                <div class="flex items-center justify-center space-x-2 text-blue-600">
                    <i class="fas fa-arrow-up animate-bounce"></i>
                    <span class="font-medium">Use the "Create Network" form above</span>
                </div>
            </div>
        </div>"#
    } else {
        ""
    };

    template
        .replace("{{NETWORK_ROWS}}", &network_rows)
        .replace("{{NETWORK_OPTIONS}}", &network_options)
        .replace("{{EMPTY_NETWORKS_MESSAGE}}", empty_networks_message)
}

pub fn network_vouchers_template(
    network: Option<&WiFiNetwork>,
    vouchers: &[Voucher],
    network_id: &str,
    voucher_counts: &VoucherCounts,
) -> String {
    let template = load_template("network-vouchers");

    let network_info = match network {
        Some(net) => format!(
            r#"
            <div class="mb-8 animate-fade-in">
                <div class="bg-white rounded-2xl shadow-lg overflow-hidden border border-gray-200">
                    <div class="bg-gradient-to-r from-blue-500 to-indigo-600 p-8 relative overflow-hidden">
                        <div class="absolute top-0 right-0 w-32 h-32 bg-white opacity-10 rounded-full -mr-16 -mt-16"></div>
                        <div class="absolute bottom-0 left-0 w-24 h-24 bg-white opacity-10 rounded-full -ml-12 -mb-12"></div>
                        <div class="relative z-10">
                            <div class="flex items-center justify-between">
                                <div>
                                    <h2 class="text-3xl font-bold text-white mb-3">
                                        <i class="fas fa-wifi mr-3"></i>{}
                                    </h2>
                                    <p class="text-blue-100 text-lg">Network Configuration Details</p>
                                </div>
                                <div class="hidden lg:block">
                                    <i class="fas fa-network-wired text-6xl text-white opacity-30"></i>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="p-8">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                            <div class="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-6 border border-blue-200">
                                <div class="flex items-center mb-3">
                                    <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-3">
                                        <i class="fas fa-broadcast-tower text-white"></i>
                                    </div>
                                    <h3 class="font-bold text-blue-900">Network SSID</h3>
                                </div>
                                <div class="font-mono bg-white text-blue-800 px-4 py-3 rounded-lg border border-blue-300 text-lg font-semibold">
                                    {}</div>
                            </div>
                            <div class="bg-gradient-to-br from-purple-50 to-indigo-50 rounded-xl p-6 border border-purple-200">
                                <div class="flex items-center mb-3">
                                    <div class="w-10 h-10 bg-gradient-to-br from-purple-500 to-purple-600 rounded-lg flex items-center justify-center mr-3">
                                        <i class="fas fa-info-circle text-white"></i>
                                    </div>
                                    <h3 class="font-bold text-purple-900">Description</h3>
                                </div>
                                <p class="text-purple-800 text-lg">{}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            "#,
            net.name,
            net.ssid,
            net.description.as_ref().unwrap_or(&"No description provided".to_string())
        ),
        None => r#"
            <div class="mb-8 animate-fade-in">
                <div class="bg-white rounded-2xl shadow-lg overflow-hidden border border-red-200">
                    <div class="bg-gradient-to-r from-red-500 to-pink-600 p-8">
                        <h2 class="text-3xl font-bold text-white mb-2">
                            <i class="fas fa-exclamation-triangle mr-3"></i>Network Not Found
                        </h2>
                        <p class="text-red-100 text-lg">The requested network could not be found in the system.</p>
                    </div>
                </div>
            </div>
        "#.to_string(),
    };

    let voucher_rows = vouchers
        .iter()
        .enumerate()
        .map(|(i, voucher)| {
            let status_badge = if voucher.is_used {
                r#"<span class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-semibold bg-gradient-to-r from-red-100 to-pink-100 text-red-800 border border-red-200">
                    <i class="fas fa-times-circle mr-1"></i>Used
                </span>"#
            } else {
                r#"<span class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-semibold bg-gradient-to-r from-green-100 to-emerald-100 text-green-800 border border-green-200">
                    <i class="fas fa-check-circle mr-1"></i>Available
                </span>"#
            };

            format!(
                r#"
                <tr class="group hover:bg-gradient-to-r hover:from-gray-50 hover:to-blue-50 transition-all duration-300 border-b border-gray-100">
                    <td class="px-6 py-6">
                        <div class="flex items-center">
                            <div class="w-8 h-8 bg-gradient-to-br from-gray-600 to-gray-700 rounded-lg flex items-center justify-center mr-3">
                                <span class="text-white text-sm font-bold">{}</span>
                            </div>
                        </div>
                    </td>
                    <td class="px-6 py-6">
                        <div class="flex items-center">
                            <div class="w-10 h-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg flex items-center justify-center mr-4">
                                <i class="fas fa-barcode text-white text-sm"></i>
                            </div>
                            <div>
                                <div class="text-sm font-mono bg-gradient-to-r from-gray-100 to-gray-200 text-gray-800 px-3 py-2 rounded-lg border border-gray-300 font-semibold">
                                    {}
                                </div>
                                <div class="text-xs text-gray-500 mt-1">Voucher Code</div>
                            </div>
                        </div>
                    </td>
                    <td class="px-6 py-6">{}</td>
                    <td class="px-6 py-6">
                        <div class="flex items-center text-sm text-gray-600">
                            <i class="fas fa-calendar-alt mr-2 text-gray-400"></i>
                            <div>
                                <div class="font-medium">{}</div>
                                <div class="text-xs text-gray-400">Created</div>
                            </div>
                        </div>
                    </td>
                </tr>
                "#,
                i + 1,
                voucher.code,
                status_badge,
                voucher.created_at.format("%Y-%m-%d %H:%M")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let empty_vouchers_message = if vouchers.is_empty() {
        r#"<div class="text-center py-16 animate-fade-in">
            <div class="bg-gradient-to-br from-amber-50 to-orange-100 rounded-2xl p-12 border border-amber-200 max-w-md mx-auto">
                <i class="fas fa-ticket-alt text-6xl text-amber-400 mb-6"></i>
                <h3 class="text-2xl font-bold text-gray-800 mb-4">No Vouchers Yet</h3>
                <p class="text-gray-600 mb-6">This network doesn't have any voucher codes uploaded yet.</p>
                <div class="space-y-3">
                    <div class="flex items-center justify-center space-x-2 text-amber-600">
                        <i class="fas fa-arrow-up animate-bounce"></i>
                        <span class="font-medium">Go to Admin Panel to upload vouchers</span>
                    </div>
                    <a href="/admin" class="inline-block bg-gradient-to-r from-amber-500 to-orange-600 hover:from-amber-600 hover:to-orange-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                        <i class="fas fa-upload mr-2"></i>Upload Vouchers
                    </a>
                </div>
            </div>
        </div>"#
    } else {
        ""
    };

    template
        .replace("{{NETWORK_ID}}", network_id)
        .replace("{{NETWORK_INFO}}", &network_info)
        .replace("{{VOUCHER_COUNT}}", &vouchers.len().to_string())
        .replace("{{VOUCHER_ROWS}}", &voucher_rows)
        .replace("{{EMPTY_VOUCHERS_MESSAGE}}", empty_vouchers_message)
        .replace("{{TOTAL_COUNT}}", &voucher_counts.total.to_string())
        .replace("{{USED_COUNT}}", &voucher_counts.used.to_string())
        .replace("{{UNUSED_COUNT}}", &voucher_counts.unused.to_string())
}
