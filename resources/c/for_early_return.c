int find(int *a, int n, int x) {
  for (int i = 0; i < n; i++) {
    if (a[i] == x) return i;
  }
  return -1;
}
int main() { int a[3]; int r = find(a, 3, 2); return 0; }
