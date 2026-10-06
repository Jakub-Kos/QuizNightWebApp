// Link to the public Google Sheets template (any link to the spreadsheet). The help page shows a
// "make a copy" button when it is set.
export const GOOGLE_SHEET_TEMPLATE = "https://docs.google.com/spreadsheets/d/1p1-bxa2EBWcRvRE_bWu3AbHaSHkgjD7jgBcvu1tWNvw/edit?usp=sharing";

// ".../spreadsheets/d/<id>/copy" opens Google's "Make a copy" dialog
export function googleCopyUrl(url) {
  const match = String(url).match(/docs\.google\.com\/spreadsheets\/d\/([\w-]+)/);
  return match ? `https://docs.google.com/spreadsheets/d/${match[1]}/copy` : url;
}
