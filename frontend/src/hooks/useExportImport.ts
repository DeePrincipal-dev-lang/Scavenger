/**
 * useExportImport
 *
 * Convenience hook that exposes all export/import utilities from
 * the unified lib/exportImport module so components can consume them
 * without importing from the lib directly.
 */
export {
  exportWasteToCSV,
  exportWasteToJSON,
  exportAnalyticsToCSV,
  exportParticipantStatsToPDF,
  PDFExporter,
  parseWasteCSV,
  generateWasteTemplateCSV,
  triggerDownload,
  type AnalyticsMonthlyRow,
  type ImportWasteRow,
  type ParsedImportResult,
  type ImportPreviewRow,
  type WasteExportData,
  type PDFExportOptions,
} from '@/lib/exportImport'
