use ilm_core::base::keys::NO_NUMBER;
use ilm_core::base::lanes::on_lanes;
use ilm_core::base::sums::compensated;
use ilm_core::base::symbols::Symbols;
use ilm_core::know::signs::{is_space, is_word_sign};
use ilm_core::write::find::WantedValue;
use ilm_core::write::fit::PlaceFiller;
use ilm_core::write::forms::SentenceForm;
use ilm_core::write::levels::Writer;
use ilm_core::write::measure::{at_even_steps, grouped_lists};
use ilm_core::write::numbers::{e_to, natural_log, to_power};
use ilm_core::write::overlap::run_overlap;
use ilm_core::write::rows::FactRow;
use ilm_core::write::signs::{first_capital, letter_or_digit, low_tokens};
use std::time::Instant;

const NAMES: [&str; 10] = ["Alba", "Bruno", "Cedar House", "Delta", "Elm Tree", "Fox and Hen", "Gino", "Harbour View", "Iris", "Juniper"];
const KINDS: [&str; 3] = ["pub", "inn", "tea room"];
const TOWNS: [&str; 4] = ["Ashby", "Brent", "Colne", "Dover"];
const GOODS: [&str; 6] = ["tea", "cake", "jam", "bread", "honey", "cheese"];
const MOST: [&str; 8] = ["oldest", "newest", "finest", "cheapest", "largest", "smallest", "nearest", "loudest"];
const PLAIN: [&str; 10] = ["quiet", "busy", "friendly", "modern", "famous", "narrow", "humble", "cosy", "sunny", "popular"];
const WORTH: [&str; 3] = ["good", "fair", "poor"];
const FOODS: [&str; 5] = ["pies", "soups", "cakes", "stews", "roasts"];
const DAYS: [&str; 3] = ["Sunday", "Monday", "Friday"];
const YEARS: [&str; 6] = ["1990", "1995", "2000", "2005", "2010", "2015"];
const STALL_WORDS: [&str; 9] = ["organic", "local", "fresh", "cheap", "sweet", "plain", "bread", "cake", "jam"];
const WARES: [&str; 3] = ["bread", "cake", "jam"];

const FLOATS: &str = "
log 3f8d2e893c5dc4e9 c0110119a529ab2a
exp bff26fc19fd66a8f 3fd437f0aa884816
pow 3fd5b22d0e560419 4014000000000000 3f72569912e71ff5
log 3fda704df54bf618 bfec4a2cf1000d32
exp c0334213d7311ef8 3e3296e695756eca
pow 3fee8c97eb2fd4b7 4014000000000000 3fe95fe440d1b01c
log 40413532899df630 400c4ede60a57221
exp 3fd587236c61ec98 3ff665d502336d88
pow 3fedf3b645a1cac1 4060600000000000 3f26a0c467fef5e2
log 40185e85e85e85e8 3ffce995150208c6
exp c048a48bf01f7658 3b7dc65eabc1cb0f
pow 3fce2d5e88c3819c 4014000000000000 3f47ddeee91fbe5d
log 403b7a656116555e 400a81d2710ce730
exp c02b50d79435e50d 3eb3a4303b9bdaeb
pow 3fee872b020c49ba 4014000000000000 3fe9496402922787
log 3fe02ce8e5bb0d51 bfe5d4ee57bcb7fc
exp c036733618c3e8de 3de871c4b9c72fed
pow 3fe45d35f91b5bb0 4049800000000000 3ddad48211e2ca60
log 3fec1881cd9f7602 bfc0a7b72e8f0d77
exp 3fe021aa6e671522 3ffa7cf8de8bcac3
pow 3fef87a57635440f 4014000000000000 3fedb7a7db770c28
log 40225e1281a4850b 4001bd4e06921050
exp c0385f417d05f418 3dbc9c7002283731
pow 3fefd81ab73fecfc 404a800000000000 3fe8b463e1aedc94
log 3fc48fcf873834a8 bffd41ef24af5bc6
exp c03d0055f6fadab6 3d51e041043239e3
pow 3fb6c8b439581062 4014000000000000 3ed76bd692d6eca3
log 40073d5eaf57abd6 3ff1101018dbe77d
exp c03487ca92ebf718 3e14d569ef22b02b
pow 3fdcfb5244c3c420 4014000000000000 3f937fd8751641af
log 3ff573ff316cbc04 3fd2c51dc80b8c77
exp c0403c2b1a262798 3d01d276fc49d991
pow 3fdeeed073a11584 4014000000000000 3f9b022f6a3266dc
log 4038ce48de4b040e 4009b04962b5efbe
exp c03131524881bc34 3e6254cbd3b3d172
pow 3fef2b020c49ba5e 4014000000000000 3fec0cff28ae335a
log 3e3af6298c16d2ac c032e2e45fa5f2e7
exp c030f7b282135f7b 3e66f572f57d662b
pow 3f9db22d0e560419 4014000000000000 3e56060fcbf92e51
log 3fe21513c8a9e6e7 bfe243ff6f2dcf53
exp c0047711dc47711e 3fb3d3ad851c31ba
pow 3fec50944116a2c8 4070f00000000000 3cf1ec59690de4d9
log 3fe7f0b45e69ddb2 bfd29238c4edf6b6
exp bfc3960d09e3b4b0 3feb75ae6cc81252
pow 3fe446745179ee84 4073400000000000 3342b8f2778b449c
log 40b7144cfbfe89a0 40215e44215a985d
exp bfd67250fa869308 3fe6889b93059f98
pow 3fcccccccccccccd 406d000000000000 20baa2b4cd2c1456
log 3f4567e42ee759dc c01d558cb74c5967
exp c02351462e550de0 3f10bde8e8d125d7
pow 3fc999999999999a 4014000000000000 3f34f8b588e368f2
log 4024d7bbe90bb1f9 4002c03828d979a5
exp c029c5abbf309b8b 3ec53fbd95809a2b
pow 3fcc3fdb540d877c 4036000000000000 3cf07cd2976fdb7c
log 4027e6ffc9028015 4003d8bcfaf433ae
exp bfea220fe7ae1cd0 3fdc483050cc7eab
pow 3fec69dc5c3258d0 4076000000000000 3c28d44bc1d5a905
log 3fd134f10c534928 bff5046aa02144ed
exp 3feaadb53f897494 40026a26015f68f9
pow 3fdf7ced916872b0 4060200000000000 37aff5c6dd0d2eef
log 402f1f3fd5a50f03 4005f549f434857b
exp c031130463796aca 3e64a2870e140d6c
pow 3fc126e978d4fdf4 4014000000000000 3f06a6be410c9ebb
log 40431688e3155c98 400d233e2b0c85c9
exp 3fe96cb25123ded4 4001b4fe17f11546
pow 3fe3ae3694dd1e28 406a600000000000 36b0477627ec0b3e
log 3ff06d54aebbe98c 3f9af96a4f5594d0
exp c046413f75293e58 3beb96b540b74c25
pow 3fdccccccccccccd 4000000000000000 3fc9eb851eb851ec
log 3ff5c0768f091830 3fd3a7a92e7dd15a
exp 3f91df8839610240 3ff0481ec9799b96
pow 3fd1af74d3d50e76 406a200000000000 27b2587ee3d3303c
log 3d91de833e252f5f c03a3aa68831e537
exp bfb8bce130b02220 3fed0d6f49c09a26
pow 3fd51eb851eb851f 4014000000000000 3f7007a4b4e1bbcc
log 3fdbf706baf3c929 bfea7e69b810dee9
exp c046d6ad92e1d70a 3bd12b5fe3d60a72
pow 3fead6de31884e41 4014000000000000 3fda904d6d8c4b80
log 3ff7787e12bbd931 3fd885be12f49c85
exp c03ee739ce739ce7 3d25593dad86908a
pow 3fed44a38e604f7a 4045000000000000 3f9821a8e3696ebb
log 403a728c75198170 400a338710d5b71e
exp c05615e50d79435e 37f769ea29223849
pow 3fe9153c3f24bef4 406f800000000000 3a65f2da8866778f
log 3fec7ee9d567e0b5 bfbdb0ea31311980
exp 3fbd150f3a2d27a0 3ff1ecc6d779972c
pow 3feda1cac083126f 4014000000000000 3fe5c9916b32d6e7
log 4048f8df9d95fc0d 400f498a96d4181a
exp c03f555555555555 3d1bc59d04078e79
pow 3fc536a2570b50fc 4062000000000000 289827f43b825c4d
log 3ffe70076828bce0 3fe494324d217f9e
exp c01021630d201c98 3f9227508cd92254
pow 3f87e4f2420e5a40 4072000000000000 0000000000000000
log 40f4927f2d072b8e 4026aef488aac2dc
exp bfc6e2aad452c0d0 3feac2cd5254e242
pow 3fbd272f27fa9458 4014000000000000 3ef41528afd3f6c0
log 40425c941f451031 400cd3c8f2e01ca7
exp bfc57ea8717d9f18 3feb0da37f61c51d
pow 3fe1bf5c97737bcd 404b000000000000 3d10dbc35e770fbc
log 400f12a6e47542c0 3ff5b5d5a456ddd8
exp 3fb996d4cce76630 3ff1ae96be19e56c
pow 3fec9374bc6a7efa 4068400000000000 3df40e8e268dca87
log 403ff24916582225 400bb66548029611
exp 3fe003157df42b7a 3ffa63b480f8ebc5
pow 3f91a65fbf801940 4014000000000000 3e1a22b16129bde9
log 3fe7a8e27fe6a84a bfd3535a60b8e36c
exp bfecfe0a55d1612a 3fd9dd517007dac6
pow 3fe6b851eb851eb8 405c800000000000 3c697c1cda17bdf8
log 3fed6fa0b48857ce bfb5614cc2cf3b91
exp c032d508f377f1ae 3e3c76118e85d176
pow 3fdcda277eafabda 4032000000000000 3ea3d9256f0fbf1d
log 3fddfa5fa5fa5fa6 bfe844f705d8092a
exp c045d83446d56c5b 3bff572e8f3c2c26
pow 3fe5c2f2cd31c4f8 4072400000000000 35c7a12678034e4d
log 4084e06ebc3ebfe4 401a0479563be1c4
exp c024228a28a28a29 3f063ff767483511
pow 3fca5e353f7ced91 4014000000000000 3f384fce27d4c14f
log 40056e4802449b5c 3fef884a3a6191f5
exp 3fd7242fe07ec28c 3ff6f838e8f57ec8
pow 3fe1455e45327296 4014000000000000 3fa772a89c2cf96a
sum 4030ae844d47a2e8 4015daf6b985b7f5,401485042b318c51,8000000000000000,3fe0c025694e3621,40164211a33d8095
sum c0aee35dc4f1a55f 3e01f04aba13cf91,401d9f21a48d2cd0,403516f386088b64,40188425bcff0205,8000000000000000,4013e5992d64c170,3dae96577b356f82,8000000000000000,8000000000000000,3fa79596a73bf580,3dcb3541e9a31aa2,4017acf092f901d8,3fecbb6047b7e63a,3dc8609c11f409d3,3e00d2a2eedcf6d5,c08f400000000000,8000000000000000,c08f400000000000,8000000000000000,8000000000000000,4017dd55921eb176,c0322d3fd26df9e2,401765b565263775,8000000000000000,3fb7ac72824092a8,c08f400000000000,c08f400000000000,3e02e2b70aaf0b7f,3fd83658442d328a,4016fc1cb06e29cb
sum c0a65bdcf7d02e94 4013f633f5c46dee,403b697d9ddd6b3e,4022830aed9fd6a0,3fd9b4af8bf7fe84,3e0f9a0ce4df8f62,4018bead79378b0b,3df4c53b59c6a16e,3fc3b99c9140935c,3fc4060c23f195d4,3dffb7d16b40a4e6,40393602ebffe0dc,3dfcd57e1ca5f249,3de43b3546053b76,3fb57726f0c96aa0,3feea8b93831154b,4015272f78f56469,c08f400000000000,403364fe75ac004a,8000000000000000,40124c8108e54bf8,c08f400000000000,3e01af7a19629e9f,401886603ca2cbc1,4005199e7bbd02a0,c08f400000000000,8000000000000000,4038e99674f57b20,3e08c02126473dad,3df9bf16df895abd,3fd47a7459b6c2ce
sum 4036d38ab727f1cd 4016099d189b1fbf,4014e72a22d31dae,4017dff4dfe619b5,40187d6ec14b7013
sum c000d7971e5086be 3fdebba2fa647cc6,401347f7e4072429,3de167f8566b0971,40338a54181a9dac,400a1094eaf01acd,c036f717f79ed1a4,40370adfaff4abdc,401851da01eb338e,c03b72be67ac548d,400485042b318c51,8000000000000000,4016c2ed96bb3998,3df049e4d04dc480,3fe057c1e3e7455c,c0328f47c3bf371c,3e063b549a676817,3fec2cfb4688c9e5
sum 4041d4a486bb289e 3fe02ff3728b9744,3fc7815e272ca568,400d4d60297139c0,4010efb99d941abb,401615f9d9d17124,c003917a8e3afaa0,3fea6572147cfb51,401ab5519310e990,c02283de61aa8564,40392e3632201fc2,8000000000000000,3fe262c5b40e217f
sum 4025d88bbb66c60f 3df610e6a324b4b2,3fc6ad185b8be940,3e089ea2537c4c67,3fdb4db73c94f8d4,3fccae31939c3d64,3da94d3936490b8d,400f80bfc95ab220,3fb7ec39799ee4e8,40184150e8e52093
sum 40308697b83418ba 4017f9d1063f119c,4015d23fb2ce8eec,4011375cd6fcab1d,3fe8b78a8630ba1f
sum 4031000414b8f31c 3fee4b83a78e653c,3fb320e6585b3d80,3dfa67502bd5cc91,8000000000000000,4018c4faac25163e,3fcb0d5640d72e2c,3fc8625809ab0e58,3e0cea706f2c1285,3dabb28f56aa88cd,c011d1a5faff5008,3de137a193559225,bffab6d71c423100,4011296e33222fe6,3fc348e2675da86c,3fef0f091c09c15b,4017dd55921eb176,40265a3cc305f90c,4027ec754729a85c,3df6c3c198761528,3feaf43770bc45d2,c02c4e7fca34f7ae,4018dbc239b0b862,8000000000000000,3e07b18e6345e42c,c027d24cdde1c6dc,8000000000000000,3dfd08e1dabe344a
sum c08fe37fadf28f2c 3dfb6c3d86d3b8a5,401303ad25a58d8b,c08f400000000000,401019f5cb2a6c50,3e032469740c30ba,3fe222cc6c03b1f9,3e10220ec8f35ad3,4018646d95a9ee43,3dfdf59ce4a13f5a,3fed901d1233f85a,c03b242a0d9ff8d0,3fe161632fd14bed,3fdde13b880b982e,8000000000000000,402b30888feb4090,3df01d775e0507ab,c03844bb6de18218
sum c0af296154b7f627 3fbbfe99f86201c0,c08f400000000000,3fbc45b0dd0e91a8,3fdc7cc9872f3f18,8000000000000000,400f9c25aaaea949,8000000000000000,c08f400000000000,3fbeb3659fc5fc50,8000000000000000,40164dc59a08b222,c08f400000000000,3e09b66f20e95f67,3fefee9632f8087a,c08f400000000000,8000000000000000
sum c08d5dcfe7643ec1 3ddee2ad071b72df,3db833ac7a3e3070,3df7048d9f6881bb,4015a0f1a3091a36,3df96fba47eeac5e,3de226e81ea5626e,4032c211ce0ff062,403c835d3a4d22e2,c08f400000000000,8000000000000000,4017dab48b1fc4fd,4028f91d91ac9834,c03761505637b066,40032ee3b77f374c,4017c0097dae9473,4010c234da4a23a6
sum c08ea4863d6724e8 3dd8dd618a2b1877,40161a1055d4a1e2,3fedbd84f410ff91,3dfb0c6791241de9,4002b0db677dc480,3fe30e62e3dea3b8,bfe6221ffb298860,40127ff8df87c7bd,c08f400000000000,3e02ce6daf1017e2,40180b899f465639,3e05db76a947b693,3fba69f215a02098,8000000000000000
sum c09f3fdaad7b7d23 c036a307a9c3c04b,40148b0ebed7ea48,3df64a07de6eafe1,4013585a48c25f59,8000000000000000,4017a1e07e65f6f0,3e0f85d9a50ee9be,4015fd19aad0fa89,3fef6714da03fb10,3e06bcbafbb10c64,c08f400000000000,8000000000000000,8000000000000000,c08f400000000000,3fd422b5f5f70352
sum c0b325799efbba3e 8000000000000000,c015bbe48c2d2b48,403ab2892aa2e332,c08f400000000000,4031b84185080df2,403a9fa100e47d38,3fdadcaba851f35a,4018b8564920b150,4010eac91da99d08,c08f400000000000,3df1e4a7e80589d8,c02ad04df1e24090,3fd5ad5ccb0471ee,c08f400000000000,3fdb5eba81301708,4035d5dbd4c39eca,c08f400000000000,8000000000000000,3fd56885722e47da,c03614e5386637fd,8000000000000000,3e0864246a302301,4017bd54e76da01c,c08f400000000000,8000000000000000,3e0a13c62feaaffe,3e0d53e1ae811d6f,403bc51f6b107dbc,3fedf3a6621e71e2
sum c0a7a4fb112db77e c08f400000000000,4015fd19aad0fa89,3e0f65101fef33c6,401554b43c3f5b7d,8000000000000000,3de33f0d40b3630d,8000000000000000,c0359e98c7519cf4,c0307800d07bd638,c08f400000000000,c08f400000000000,3dcd63e4ce3d5eb9,3fe893b0e51ba4ad
sum c09f090c396e794e bffb73f3ff6ea6c0,3fc0be9aadd7c294,3f91a7104e5bef80,40184150e8e52093,401411b1ce380053,3dedd8ff8e4aaae3,3e1078208b08a9f6,3e0de7ad049f3a87,40182e15df7fb939,40158503cfd52c43,4017ef8cfe0b98f2,3fc6dba46939f650,c08f400000000000,8000000000000000,4018090652dcff84,8000000000000000,8000000000000000,3fe439a5b0620956,3fdd4aa39f04a752,c08f400000000000,3fe9c830077ae078,3dbfb2b52625df27,3fd3d2062d002724,c035939aa0ba1df8,3e0375236fa5a5ba
sum c0a6db4299f73ff4 3dfa14692f550319,3df9a6ec31eeeac7,3fc5938441f33a9c,8000000000000000,8000000000000000,403d0d6e2effc6be,3fd1bec52dd5d1ce,c033b6bd9df27118,40361aa88bc2e58e,3dee720782ee5d74,8000000000000000,40096ca77c922cf9,c08f400000000000,4017cac9af27fa75,bfdb0441aa4dac80,4013e82ce3e2625c,3e0845dcde9ba8ca,c08f400000000000,3febcd6975b3f092,3de86c19f1ad791c,3fd0b6c28c2b05e8,4018cd5246e43ecc,40122c5577a7bf99,c08f400000000000,4030ed45f71fa01e
sum c0a0304d01b3e978 8000000000000000,3e0fd2b87250ca79,3df0b8888fd4e5b4,3fd37a43737f7886,40032ee3b77f374c,8000000000000000,8000000000000000,c035cee716e92924,c08f400000000000,4016e199690dcf5a,4016f24002234038,c033fbad0440f671,8000000000000000,c036a013b192d87c,c08f400000000000,3fdba7c0e43f47e0,3e0f3c065446318e,c0277dcc8e0ff56c,3ddfce421a3a3847,8000000000000000,8000000000000000,c0324ccb806c0078,3fd29e252645a0f0,40125701b6f8746a,40096ca77c922cf9,c01d465b82299c10,3fea943fc0e80338,40189e931ec5491f
sum c0a7496159a551f8 8000000000000000,3fefc996fd3a8d18,3fd71c87ed1b8c3e,c08f400000000000,401576c26a9e8e99,4017ecf7d8d4aba7,c032c520db1873ad,3da58e463fb4183a,8000000000000000,3fddb313a61424c2,3dc1f0cfd27478df,401660fc0517cbb7,40153baa86fb016f,3fce8a70c3a7a694,3df0ce6d9020719d,3fe3e58ea1b78638,c08f400000000000,3fee342fc82de1ec,4016adf543c1a99a,3feb4e2e64522ada,4016bbfc55a3b52c,3e0e1c526c00277e,c08f400000000000
";

const SIGNS: &str = "
10 0 1 0 10 10
32 0 1 0 32 32
33 0 0 0 33 33
36 0 0 0 36 36
38 0 0 0 38 38
39 0 0 0 39 39
40 0 0 0 40 40
41 0 0 0 41 41
43 0 0 0 43 43
44 0 0 0 44 44
45 0 0 0 45 45
46 0 0 0 46 46
47 0 0 0 47 47
48 1 0 1 48 48
49 1 0 1 49 49
50 1 0 1 50 50
51 1 0 1 51 51
52 1 0 1 52 52
53 1 0 1 53 53
54 1 0 1 54 54
55 1 0 1 55 55
56 1 0 1 56 56
57 1 0 1 57 57
58 0 0 0 58 58
59 0 0 0 59 59
63 0 0 0 63 63
65 1 0 1 97 65 97
66 1 0 1 98 66 98
67 1 0 1 99 67 99
68 1 0 1 100 68 100
69 1 0 1 101 69 101
70 1 0 1 102 70 102
71 1 0 1 103 71 103
72 1 0 1 104 72 104
73 1 0 1 105 73 105
74 1 0 1 106 74 106
75 1 0 1 107 75 107
76 1 0 1 108 76 108
77 1 0 1 109 77 109
78 1 0 1 110 78 110
79 1 0 1 111 79 111
80 1 0 1 112 80 112
81 1 0 1 113 81 113
82 1 0 1 114 82 114
83 1 0 1 115 83 115
84 1 0 1 116 84 116
85 1 0 1 117 85 117
86 1 0 1 118 86 118
87 1 0 1 119 87 119
88 1 0 1 120 88 120
89 1 0 1 121 89 121
90 1 0 1 122 90 122
92 0 0 0 92 92
95 1 0 0 95 95
97 1 0 1 97 65 65
98 1 0 1 98 66 66
99 1 0 1 99 67 67
100 1 0 1 100 68 68
101 1 0 1 101 69 69
102 1 0 1 102 70 70
103 1 0 1 103 71 71
104 1 0 1 104 72 72
105 1 0 1 105 73 73
106 1 0 1 106 74 74
107 1 0 1 107 75 75
108 1 0 1 108 76 76
109 1 0 1 109 77 77
110 1 0 1 110 78 78
111 1 0 1 111 79 79
112 1 0 1 112 80 80
113 1 0 1 113 81 81
114 1 0 1 114 82 82
115 1 0 1 115 83 83
116 1 0 1 116 84 84
117 1 0 1 117 85 85
118 1 0 1 118 86 86
119 1 0 1 119 87 87
120 1 0 1 120 88 88
121 1 0 1 121 89 89
122 1 0 1 122 90 90
163 0 0 0 163 163
233 1 0 1 233 201
";

struct Mix {
    state: u64,
}

impl Mix {
    fn new(start: u64) -> Mix {
        Mix { state: start }
    }

    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

fn facts(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs.iter().map(|(slot, value)| (slot.to_string(), value.to_string())).collect()
}

fn row(pairs: &[(&str, &str)], text: &str) -> FactRow {
    FactRow { facts: facts(pairs), text: text.to_string() }
}

fn places() -> Vec<FactRow> {
    (0..240)
        .map(|n: usize| {
            let (name, kind, town, dogs) = (NAMES[n % 10], KINDS[n % 3], TOWNS[(n / 3) % 4], ["yes", "no"][(n / 5) % 2]);
            let mut pairs = vec![("name", name), ("kind", kind)];
            let mut text = [format!("{name} is a {kind}."), format!("The {kind} {name} is open."), format!("There is a {kind} called {name}.")][n % 3].clone();
            if n % 2 == 1 {
                pairs.push(("town", town));
                text += &[format!(" It is in {town}."), format!(" You find it in {town}.")][(n / 2) % 2];
            }
            if n % 4 > 1 {
                pairs.push(("dogs", dogs));
                text += if dogs == "yes" { [" Dogs are welcome.", " It takes dogs."][(n / 4) % 2] } else { [" Dogs are not allowed.", " It takes no dogs."][(n / 4) % 2] };
            }
            row(&pairs, &text)
        })
        .collect()
}

fn shops(n: usize) -> Vec<FactRow> {
    let mut mix = Mix::new(3);
    let mut out = Vec::new();
    while out.len() < n {
        let name = NAMES[mix.below(10)];
        let mut goods: Vec<&str> = GOODS.iter().copied().filter(|_| mix.below(3) == 0).collect();
        goods.truncate(3);
        if goods.is_empty() {
            continue;
        }
        let act = ["tell", "ask"][mix.below(2)];
        let said = match goods.len() {
            1 => goods[0].to_string(),
            2 => goods.join(" and "),
            _ => format!("{}, and {}", goods[..goods.len() - 1].join(", "), goods[goods.len() - 1]),
        };
        let text = if act == "tell" { format!("{name} sells {said}.") } else { format!("Does {name} sell {said}?") };
        out.push(row(&[("act", act), ("name", name), ("goods", &goods.join(", "))], &text));
    }
    out
}

fn marks() -> Vec<FactRow> {
    let mut mix = Mix::new(5);
    let mut out = Vec::new();
    for _ in 0..120 {
        let kind = KINDS[mix.below(3)];
        if mix.below(2) == 1 {
            let mark = MOST[mix.below(MOST.len())];
            out.push(row(&[("kind", kind), ("mark", mark)], &format!("What is the {mark} {kind} of the town?")));
        } else {
            let mark = PLAIN[mix.below(PLAIN.len())];
            out.push(row(&[("kind", kind), ("mark", mark)], &format!("What is the most {mark} {kind} of the town?")));
        }
    }
    out
}

fn worth() -> Vec<FactRow> {
    let mut mix = Mix::new(11);
    let mut out = Vec::new();
    for _ in 0..300 {
        let (name, kind, town, value) = (NAMES[mix.below(10)], KINDS[mix.below(3)], TOWNS[mix.below(4)], WORTH[mix.below(3)]);
        let way = mix.below(10);
        let other = match value {
            "good" => "fine",
            "fair" => "plain",
            _ => "weak",
        };
        let text = if way == 0 {
            format!("{name} in {town} is a {other} {kind} and it is open all week.")
        } else if way < 4 && value == "good" {
            format!("People in {town} are glad that {name} is a {value} {kind} and it is open all week.")
        } else if way < 7 {
            format!("People in {town} say that {name} is a {value} {kind} for a meal with friends.")
        } else {
            format!("{name} in {town} is a {value} {kind} and it is open all week.")
        };
        out.push(row(&[("name", name), ("kind", kind), ("town", town), ("worth", value)], &text));
    }
    out
}

fn asked(days: bool) -> Vec<FactRow> {
    let mut mix = Mix::new(7);
    let mut out = Vec::new();
    for _ in 0..200 {
        let (kind, town, food, name, day) = (KINDS[mix.below(3)], TOWNS[mix.below(4)], FOODS[mix.below(5)], NAMES[mix.below(10)], DAYS[mix.below(3)]);
        let way = mix.below(4);
        if way == 0 && days {
            out.push(row(&[("act", "ask"), ("kind", kind), ("day", day)], &format!("Do you know a good {kind} for {day}?")));
        } else if way < 2 {
            out.push(row(&[("act", "ask"), ("kind", kind), ("town", town)], &format!("Do you know a good {kind} in {town}?")));
        } else {
            out.push(row(&[("act", "check"), ("name", name), ("kind", kind), ("food", food)], &format!("Is {name} a good {kind} for {food}?")));
        }
    }
    out
}

fn dated() -> Vec<FactRow> {
    let mut mix = Mix::new(13);
    let mut out = Vec::new();
    for _ in 0..240 {
        let (name, kind, year) = (NAMES[mix.below(10)], KINDS[mix.below(3)], YEARS[mix.below(6)]);
        match mix.below(4) {
            0 => out.push(row(&[("act", "tell"), ("name", name), ("kind", kind)], &format!("{name} is a {kind}."))),
            1 => out.push(row(&[("act", "tell"), ("name", name), ("kind", kind), ("year", year)], &format!("{name} is a {kind}. It opened in {year}."))),
            _ => out.push(row(&[("act", "praise"), ("name", name), ("kind", kind), ("year", year)], &format!("I like the {kind} {name}, open since {year}."))),
        }
    }
    out.push(row(&[("act", "praise"), ("name", "Alba"), ("kind", "inn"), ("year", "1990")], "I like the inn Alba, a relic of the far days of 1990."));
    out
}

fn stalls(n: usize) -> Vec<FactRow> {
    let mut mix = Mix::new(17);
    let mut out = Vec::new();
    while out.len() < n {
        let name = NAMES[mix.below(10)];
        let first = STALL_WORDS[mix.below(9)];
        let second = if mix.below(2) == 1 { Some(STALL_WORDS[mix.below(9)]) } else { None };
        let items: Vec<&str> = match second {
            Some(second) if second != first => vec![first, second],
            _ => vec![first],
        };
        let said = items.join(" ");
        let plain_end = WARES.contains(&items[items.len() - 1]) && mix.below(2) == 1;
        let text = if plain_end { format!("{name} sells {said}.") } else { format!("{name} sells {said} goods.") };
        out.push(row(&[("name", name), ("wares", &items.join(", "))], &text));
    }
    out
}

fn model(rows: &[FactRow]) -> Writer {
    Writer::learn(rows).unwrap()
}

fn slot(writer: &Writer, name: &str) -> u32 {
    writer.lessons.slots.find(name).unwrap()
}

fn unit(writer: &Writer, text: &str) -> u32 {
    writer.lessons.syms.find(text).unwrap()
}

fn lesson_form<'a>(writer: &'a Writer, word: &str) -> &'a SentenceForm {
    let word = unit(writer, word);
    writer.forms.forms[..writer.forms.lesson_forms].iter().find(|form| form.key.contains(&word)).unwrap()
}

#[test]
fn a_name_that_no_lesson_has() {
    let writer = model(&places());
    assert_eq!(writer.write(&facts(&[("name", "Zum Ochsen"), ("kind", "tea room"), ("town", "Dover")])), "Zum Ochsen is a tea room. It is in Dover.");
}

#[test]
fn every_fact_is_said_and_the_trace_names_the_lesson_forms() {
    let writer = model(&places());
    let given = facts(&[("name", "Zum Ochsen"), ("kind", "pub"), ("town", "Brent"), ("dogs", "no")]);
    assert_eq!(writer.write(&given), "Zum Ochsen is a pub. You find it in Brent. It takes no dogs.");
    let trace = writer.desk().trace(&given);
    assert!(!trace.contains("NOT SAID"));
    assert!(trace.contains("lessons"));
}

#[test]
fn a_slot_whose_value_stands_word_for_word_is_a_name_slot() {
    let writer = model(&places());
    let mut names: Vec<&str> = (0..writer.lessons.slots.len() as u32).filter(|slot| writer.lessons.is_name(*slot)).map(|slot| writer.lessons.slots.name(slot)).collect();
    names.sort();
    assert_eq!(names, ["kind", "name", "town"]);
}

#[test]
fn inputs_of_unusual_shape_give_a_text_and_do_not_stop() {
    let base: Vec<FactRow> = places()[..60].to_vec();
    let some: Vec<Vec<(String, String)>> = base[..3].iter().map(|row| row.facts.clone()).collect();
    let mut odd_slot = some[0].clone();
    odd_slot.push(("parking".to_string(), "yes".to_string()));
    let empty_value = vec![some[0][0].clone(), ("kind".to_string(), String::new())];
    let no_token = vec![some[0][0].clone(), ("kind".to_string(), "   ".to_string())];
    let mut empty_text = base.clone();
    empty_text.push(FactRow { facts: some[0].clone(), text: String::new() });
    let mut no_facts = base.clone();
    no_facts.push(row(&[], "Hello there."));
    let cases: Vec<(Vec<FactRow>, Vec<Vec<(String, String)>>)> = vec![
        (Vec::new(), some.clone()),
        (base[..3].to_vec(), some.clone()),
        (base.clone(), vec![Vec::new()]),
        (base.clone(), vec![odd_slot]),
        (base.clone(), vec![empty_value]),
        (base.clone(), vec![no_token]),
        (empty_text, some.clone()),
        (no_facts, some.clone()),
    ];
    for (rows, lists) in cases {
        let writer = model(&rows);
        for given in lists {
            let _ = writer.write(&given);
            let _ = writer.desk().trace(&given);
        }
    }
}

#[test]
fn two_models_and_any_number_of_threads_write_the_same_texts() {
    let rows = places();
    let (one, two) = (model(&rows), model(&rows));
    let lists: Vec<Vec<(String, String)>> = rows[..60].iter().map(|row| row.facts.clone()).collect();
    let alone: Vec<String> = on_lanes(lists.len(), 1, &|at| one.write(&lists[at]));
    let together: Vec<String> = on_lanes(lists.len(), 4, &|at| two.write(&lists[at]));
    assert_eq!(alone, together);
}

#[test]
fn what_is_counted_for_lists_and_sorts_of_text() {
    let writer = model(&shops(200));
    let goods = slot(&writer, "goods");
    for number in 0..writer.lessons.slots.len() as u32 {
        assert_eq!(writer.lessons.cut(number), if number == goods { Some(", ") } else { None });
        assert_eq!(writer.reader.carried[number as usize], writer.lessons.slots.name(number) == "act");
    }
    let mut gaps: Vec<Vec<&str>> = writer.forms.list_gaps.iter().filter(|(_, count)| **count >= 2).map(|(gap, _)| gap.iter().map(|unit| writer.lessons.syms.name(*unit)).collect()).collect();
    gaps.sort();
    assert_eq!(gaps, [vec![","], vec![",", "and"]]);
}

#[test]
fn a_list_of_four_items_from_lessons_with_three_at_most() {
    let writer = model(&shops(200));
    let given = facts(&[("act", "tell"), ("name", "Zum Ochsen"), ("goods", "tea, cake, jam, bread")]);
    assert_eq!(writer.write(&given), "Zum Ochsen sells tea, cake, jam, and bread.");
    assert!(writer.desk().trace(&given).contains("written again for 4 items of goods"));
}

#[test]
fn the_sort_of_text_chooses_the_form_and_the_counts_choose_the_order() {
    let writer = model(&shops(200));
    assert_eq!(writer.write(&facts(&[("act", "ask"), ("name", "Zum Ochsen"), ("goods", "honey, cheese")])), "Does Zum Ochsen sell honey and cheese?");
    assert_eq!(writer.write(&facts(&[("act", "tell"), ("name", "Zum Ochsen"), ("goods", "bread")])), "Zum Ochsen sells bread.");
    assert_eq!(writer.write(&facts(&[("act", "tell"), ("name", "Zum Ochsen"), ("goods", "honey, bread, jam, cake, tea")])), "Zum Ochsen sells tea, bread, jam, cake, and honey.");
    let two = writer.write(&facts(&[("act", "tell"), ("name", "Zum Ochsen"), ("goods", "jam, tea")]));
    assert!(two == "Zum Ochsen sells tea and jam." || two == "Zum Ochsen sells jam and tea.");
}

#[test]
fn a_name_goes_where_names_of_its_form_stand() {
    let writer = model(&marks());
    let mark = slot(&writer, "mark");
    assert!(writer.lessons.is_name(mark));
    let mut filler = PlaceFiller::new(&writer.lessons, &writer.reader, &writer.forms);
    let grandest = writer.lessons.value_number("grandest", &mut filler.fresh);
    let calm = writer.lessons.value_number("calm", &mut filler.fresh);
    let paced = writer.lessons.value_number("fast-paced", &mut filler.fresh);
    assert_eq!(filler.end_main(mark, grandest), unit(&writer, "the"));
    assert_eq!(filler.end_main(mark, calm), NO_NUMBER);
    assert_eq!(filler.end_main(mark, paced), NO_NUMBER);
    assert_eq!(writer.write(&facts(&[("kind", "pub"), ("mark", "grandest")])), "What is the grandest pub of the town?");
    assert_eq!(writer.write(&facts(&[("kind", "inn"), ("mark", "calm")])), "What is the most calm inn of the town?");
    assert_eq!(writer.write(&facts(&[("kind", "inn"), ("mark", "fast-paced")])), "What is the most fast-paced inn of the town?");
}

#[test]
fn a_form_with_a_word_of_one_value_serves_that_value_alone() {
    let writer = model(&worth());
    let form = lesson_form(&writer, "glad");
    let worth_slot = slot(&writer, "worth");
    assert!(form.key.contains(&writer.lessons.labels[worth_slot as usize]));
    let seen = &writer.forms.values_with[worth_slot as usize][&unit(&writer, "glad")];
    assert_eq!(seen.rows().iter().map(|(value, _)| writer.lessons.vals.name(*value)).collect::<Vec<&str>>(), ["good"]);
    let mut filler = PlaceFiller::new(&writer.lessons, &writer.reader, &writer.forms);
    let (good, poor) = (writer.lessons.vals.find("good").unwrap(), writer.lessons.vals.find("poor").unwrap());
    assert!(filler.other_value(form, worth_slot, poor));
    assert!(!filler.other_value(form, worth_slot, good));
    let base = [("name", "Zum Ochsen"), ("kind", "inn"), ("town", "Dover")];
    for (value, fits) in [("poor", false), ("good", true)] {
        let mut pairs = base.to_vec();
        pairs.push(("worth", value));
        let numbered = writer.lessons.facts_of(&facts(&pairs), &mut filler.fresh);
        let given = writer.lessons.given_of(&numbered, &mut filler.fresh);
        assert_eq!(filler.filled(form, &given, 0).is_some(), fits);
    }
    for value in ["fair", "poor"] {
        let mut pairs = base.to_vec();
        pairs.push(("worth", value));
        let text = writer.write(&facts(&pairs));
        assert!(text.contains(value) && !text.contains("glad"), "{text}");
    }
}

#[test]
fn a_mix_of_slots_that_no_lesson_of_the_sort_has_gets_a_changed_lesson_form() {
    let writer = model(&asked(true));
    let given = facts(&[("act", "ask"), ("kind", "pub"), ("food", "pies")]);
    assert_eq!(writer.write(&given), "Do you know a good pub for pies?");
    let trace = writer.desk().trace(&given);
    assert!(trace.contains("a lesson form changed: taken out <") && trace.contains("[food]>") && !trace.contains("NOT SAID"), "{trace}");
    let plain = facts(&[("act", "ask"), ("kind", "inn"), ("town", "Dover")]);
    assert_eq!(writer.write(&plain), "Do you know a good inn in Dover?");
    assert!(!writer.desk().trace(&plain).contains("changed"));
    let without = model(&asked(false));
    assert!(!without.write(&given).contains("for pies"));
    assert!(!without.desk().trace(&given).contains("changed"));
}

#[test]
fn a_form_with_a_word_of_one_year_serves_that_year_alone() {
    let writer = model(&dated());
    let form = lesson_form(&writer, "relic");
    let year = slot(&writer, "year");
    let mut filler = PlaceFiller::new(&writer.lessons, &writer.reader, &writer.forms);
    let (late, early) = (writer.lessons.vals.find("2015").unwrap(), writer.lessons.vals.find("1990").unwrap());
    assert!(filler.one_value(form, year, late));
    assert!(!filler.one_value(form, year, early));
    for (value, fits) in [("2015", false), ("1990", true)] {
        let numbered = writer.lessons.facts_of(&facts(&[("act", "praise"), ("name", "Zum Ochsen"), ("kind", "pub"), ("year", value)]), &mut filler.fresh);
        let given = writer.lessons.given_of(&numbered, &mut filler.fresh);
        assert_eq!(filler.filled(form, &given, 0).is_some(), fits);
    }
    assert_eq!(writer.write(&facts(&[("act", "praise"), ("name", "Zum Ochsen"), ("kind", "pub"), ("year", "2015")])), "I like the pub Zum Ochsen, open since 2015.");
}

#[test]
fn a_list_does_not_end_with_a_word_that_the_lessons_never_end_it_with() {
    let writer = model(&stalls(320));
    let wares = slot(&writer, "wares");
    let mut filler = PlaceFiller::new(&writer.lessons, &writer.reader, &writer.forms);
    let (organic, bread) = (writer.lessons.vals.find("organic").unwrap(), writer.lessons.vals.find("bread").unwrap());
    assert!(!filler.closes(wares, organic, unit(&writer, ".")));
    assert!(filler.closes(wares, bread, unit(&writer, ".")));
    assert!(filler.closes(wares, organic, unit(&writer, "goods")));
    for items in ["local", "fresh, organic"] {
        assert!(writer.write(&facts(&[("name", "Zum Ochsen"), ("wares", items)])).ends_with("goods."), "{items}");
    }
    assert_eq!(writer.write(&facts(&[("name", "Zum Ochsen"), ("wares", "jam")])), "Zum Ochsen sells jam.");
}

#[test]
fn the_measure_counts_runs_lists_and_steps() {
    assert_eq!(low_tokens("Alba is a pub."), low_tokens("alba is a pub ."));
    let mut numbers = Symbols::new();
    let mut numbered = |text: &str| -> Vec<u32> { low_tokens(text).iter().map(|token| numbers.number(token)).collect() };
    let text = numbered("Alba is a pub in Dover and it takes dogs.");
    let other = numbered("Nothing here matches that one sentence at all, no.");
    assert!((run_overlap(std::slice::from_ref(&text), &[vec![text.clone()]]) - 100.0).abs() < 1e-9);
    assert_eq!(run_overlap(&[text], &[vec![other]]), 0.0);
    let rows = [row(&[("name", "Alba")], "Alba."), row(&[("name", "Iris")], "Iris."), row(&[("name", "Alba")], "It is Alba.")];
    let lists = grouped_lists(&rows);
    assert_eq!(lists.len(), 2);
    assert_eq!(lists[0].references, ["Alba.", "It is Alba."]);
    assert_eq!(lists[1].references, ["Iris."]);
    let steps: Vec<FactRow> = (0..10).map(|n| row(&[], &n.to_string())).collect();
    assert_eq!(at_even_steps(&steps, 5).iter().map(|row| row.text.as_str()).collect::<Vec<&str>>(), ["0", "2", "4", "6", "8"]);
}

#[test]
fn numbers_have_the_bits_of_the_table() {
    let number = |bits: &str| f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
    let mut checked = 0;
    for line in FLOATS.lines().filter(|line| !line.is_empty()) {
        let parts: Vec<&str> = line.split(' ').collect();
        let (got, want) = match parts[0] {
            "log" => (natural_log(number(parts[1])), number(parts[2])),
            "exp" => (e_to(number(parts[1])), number(parts[2])),
            "pow" => (to_power(number(parts[1]), number(parts[2])), number(parts[3])),
            _ => (compensated(parts[2].split(',').map(number)), number(parts[1])),
        };
        assert_eq!(got.to_bits(), want.to_bits(), "{line}");
        checked += 1;
    }
    assert!(checked > 100);
}

#[test]
fn signs_have_the_classes_of_the_table() {
    let code = |text: &str| char::from_u32(text.parse::<u32>().unwrap()).unwrap();
    let run = |text: &str| -> String { text.split('.').filter(|part| !part.is_empty()).map(code).collect() };
    let rows: Vec<Vec<&str>> = SIGNS.lines().filter(|line| !line.is_empty()).map(|line| line.split(' ').collect()).collect();
    let all: Vec<char> = rows.iter().map(|row| code(row[0])).collect();
    for row in &rows {
        let sign = code(row[0]);
        let own = sign.to_string();
        assert_eq!(is_word_sign(sign), row[1] == "1", "{}", row[0]);
        assert_eq!(is_space(sign), row[2] == "1", "{}", row[0]);
        assert_eq!(letter_or_digit(&own), row[3] == "1", "{}", row[0]);
        assert_eq!(own.to_lowercase(), run(row[4]), "{}", row[0]);
        assert_eq!(first_capital(&own), run(row[5]), "{}", row[0]);
        let pattern = WantedValue::new(&own);
        let same: String = all.iter().filter(|other| **other != sign && pattern.stands(&other.to_string())).collect();
        assert_eq!(same, run(row.get(6).copied().unwrap_or("")), "{}", row[0]);
    }
    assert!(rows.len() > 50);
}

#[test]
fn speed_of_the_model_and_of_the_texts() {
    let rows = stalls(2000);
    let lists: Vec<Vec<(String, String)>> = shops(400).iter().map(|row| row.facts.clone()).chain(stalls(400).iter().map(|row| row.facts.clone())).collect();
    let started = Instant::now();
    let writer = model(&rows);
    let made = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let texts: Vec<String> = on_lanes(lists.len(), 1, &|at| writer.write(&lists[at]));
    let wrote = started.elapsed().as_secs_f64();
    println!("writer: {:.0} lessons a second for the model, {:.0} texts a second on one thread", rows.len() as f64 / made, texts.len() as f64 / wrote);
    assert_eq!(texts.len(), lists.len());
}
