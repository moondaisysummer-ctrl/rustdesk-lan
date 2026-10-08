import 'package:flutter_hbb/common/widgets/autocomplete.dart';
import 'package:flutter_hbb/models/peer_model.dart';
import 'package:flutter_test/flutter_test.dart';

Peer _peer({
  required String id,
  String alias = '',
  String username = '',
  String hostname = '',
  bool online = false,
}) {
  final peer = Peer(
    id: id,
    username: username,
    hostname: hostname,
    alias: alias,
    platform: '',
    tags: [],
    hash: '',
    password: '',
    forceAlwaysRelay: false,
    rdpPort: '',
    rdpUsername: '',
    loginName: '',
    device_group_name: '',
    note: '',
  );
  peer.online = online;
  return peer;
}

void main() {
  test('merged autocomplete peers keep metadata and online state', () {
    final peers = mergeAutocompletePeers(
      lanPeers: [
        _peer(id: '123456789', alias: 'Office PC', username: 'ab-user'),
      ],
      recentPeers: [
        _peer(id: '123456789', username: 'lan-user', online: true),
      ],
    );

    expect(peers, hasLength(1));
    expect(peers.single.id, '123456789');
    expect(peers.single.alias, 'Office PC');
    expect(peers.single.username, 'ab-user');
    expect(peers.single.online, isTrue);
  });

  test('peer copies preserve online state', () {
    final peer = _peer(id: '987654321', online: true);

    expect(Peer.copy(peer).online, isTrue);
  });

}
