int main() {
  int x = 0;
again:
  x = x > 5 ? 0 : x + 1;
  if (x < 3) goto again;
  while (1) { if (x) break; }
  return 0;
}
