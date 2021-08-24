```mermaid
stateDiagram-v2
    Null --> Registered: A
    Registered --> Null: B
    Registered --> Attested: C
    Registered --> Unknown: D
    Registered --> Offline: E
    Attested --> Null: F
    Attested --> Registered: G
    Attested --> Instantiated: H
    Attested --> Unknown: I
    Attested --> Offline: J
    Instantiated --> Attested: K
    Instantiated --> Degraded: L
    Instantiated --> Unknown: M
    Degraded --> Registered: N
    Degraded --> Instantiated: O
    Degraded --> Unknown: P
    Unknown --> Null: Q
    Offline --> Null: R
    Offline --> Registered: S
```
Transition | Trigger
--- | ---
A | provider_register_geode()
B | remove_geode()<br>timeout(being Registered)
C | attestor_attest_geode()
D | report_misconduct()
E | provider_offline_geode()
F | remove_geode()
G | attestor_exit()<br>min_att_num⬆
H | provider_confirm_dispatch()
I | report_miconduct()<br>timeout(provider_confirm_dispatch())
J | provider_offline_geode()
K | provider_uninstantiate_geode()
L | attestor_exit()<br>min_att_num⬇
M | report_misconduct()<br>timeout(provider_start_serving())
N | provider_uninstantiate_geode()
O | attestor_attest_geode()<br>min_att_num⬆
P | report_misconduct()<br>timeout(provider_start_serving())<br>timeout(being Degraded) && !DegradeMode
Q | remove_geode()<br>timeout(being Unknown)
R | remove_geode()
S | provider_online_geode()

