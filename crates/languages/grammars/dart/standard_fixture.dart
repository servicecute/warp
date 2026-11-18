import 'dart:async';

/// A small fixture covering the standard Dart highlight query.
@immutable
class Consultant<T extends Object> {
  final String name;
  final int years;

  const Consultant({required this.name, this.years = 0});

  Future<String> describe() async {
    final result = await Future.value('$name has $years years');
    return result;
  }
}

void main() {
  final consultant = Consultant<String>(name: 'Ada', years: 7);
  const active = true;
  print(consultant.describe());
  if (active) return;
}
