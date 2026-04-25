export function extractFileExtension(fileName: string): string {
  if (!fileName.includes('.')) return 'bin'
  return fileName.split('.').pop()?.toLowerCase() || 'bin'
}
