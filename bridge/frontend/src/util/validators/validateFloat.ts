const floatRegEx = /^\d+?([.]|[.]\d+)?$/;

export function validateFloat(value: string): string | undefined {
  return floatRegEx.test(value) ? undefined : 'Enter a valid number'
}
