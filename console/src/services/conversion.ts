export function bytesToGB(value: number) {
  return `${(value / 1024 ** 3).toFixed(2)} Go`;
}
