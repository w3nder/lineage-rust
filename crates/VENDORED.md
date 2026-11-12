# Crates vendorizados do CanastraEngine

Origem: https://github.com/nascimentolh/CanastraEngine, commit 8fe6cf90cbbdaf890d2822a099087a9bf293da9f (2026-09-17 20:11:32 -0300).
Licença: Canastra Source License 1.0 (arquivo `LICENSE` de cada crate copiado). Uso, modificação e
redistribuição gratuita permitidos; venda do software proibida — o cliente Killer é gratuito.

Crates: `ue2-core`, `ue2-package`, `ue2-assets`, `ue2-level`, `l2-crypto`, `l2-catalog`.

Adição de metadados para LOD: `ue2-assets` conserva `SkeletalMesh.material_texture_indices`
e `textures`, já lidos do ULodMesh; `l2-catalog` conserva os caminhos em `Skinned.textures`.
Isso transporta o índice original de textura de cada slot sem usar a posição da seção.
Evidência: `docs/evidence/ui-video-app.md`, GetMaterial106799d0/slot1181067ec80.

Alterações locais (mantidas mínimas; descrever cada uma aqui):
- `l2-catalog::ColorOp`: sete operações aritméticas D3D adicionais necessárias
  ao L2ColorModifier, com transporte pelo cache e implementação WGSL;
  `docs/evidence/l2-color-stage.md`. Não habilita sozinho a classe no catálogo.
  Acrescentados também DISABLE/PREMODULATE, MULTIPLYADD com Arg0=CURRENT e
  DOTPRODUCT3; o estado bruto preserva ColorArg0 sem reescrevê-lo no auxiliar.
- `l2-catalog`: `l2_color_stage` porta a transformação de estado de `L2ColorModifier`
  (`d3ddrv.dll`, `10004fe0`) e os blocos dos chamadores shader/simples, preservando
  operações D3D brutas. Ligado a `check_material`, aos chamadores e à entrada
  `l2_modified_material` com snapshot explícito; defaults AlphaBlend/RenderTwoSided
  das duas classes irmãs corrigidos pelos defaults de Engine.u. Operações que leem
  implicitamente a textura nula continuam recusadas, sem efeito substituto.
  Comparador executa os corpos originais. Falta ligar snapshots do ator runtime.
  Evidência em `docs/evidence/l2-color-stage.md`.
- `ue2-assets`: `Property::names` decodifica arrays compactos de nomes para os vínculos `L2MovementTag`; `ue2-level::Actor` guarda também `L2MoveEvent`, e `movers::read_tags` converte Name antigo para elemento de Array como `UStruct::SerializeTaggedProperties` (core.dll, 0x10135250).
- `ue2-level`: `movers.rs` lê `MovableStaticMeshActor` e `Mover` do C4, com os estados de `physL2Movement`, `InterpolateTo`, `physMovingBrush` e troca de grau de dano. Evidência em `docs/evidence/task10-movers.md`; o formato anterior `L2MovableStaticMeshActor` permanece separado.
- `Cargo.toml` de cada crate: `version`/`edition`/`rust-version`/`license` explícitos em vez de `workspace = true`; seção `[lints]` removida.
- `l2-catalog`: pasta `Maps` (`.unr`) adicionada ao índice de arquivos para resolver heightmaps e texturas embutidas nos mapas.
- `l2-catalog`: `Catalog::open` lê a raiz do cliente uma vez e casa as pastas sem distinguir maiúsculas (`animations`, `textures`, `systextures`, `staticmeshes`, `maps` do C4 são minúsculas).
- ue2-level: Actor.skins passa a Vec<Option<String>> (entradas Null preservadas) para manter o alinhamento por seção.
- `ue2-assets/src/meshes/skeletal/{mod,lod}.rs`: leitura dos `.ukx` do C4 (versão 123, licensee 27) ao lado do caminho 37: `check_layout` devolve o `Layout`; no 27 a seção não tem bone map, o LOD não tem `LineageWedges` e o `SkeletalMesh` termina um i32 antes; os wedges vêm do fluxo empacotado `SkinningData` + `SkinPoints`, e os LODs rígidos tomam os pesos das tabelas antigas (ou `Bip01_head` sem elas). Evidência em `docs/evidence/ukx-c4.md`. `read_skeletal_mesh_lod` permite escolher o nível conservado; a entrada anterior continua escolhendo0. `section_material_slots` conserva o índice u16 original de cada seção do nível escolhido. Ainda não implementa a seleção automática do Render.
- `ue2-assets`: `l2-crypto` como dev-dependency e `tests/c4_umodel.rs`, comparação com as exportações do `umodel` (ignorada; `KILLER_CLIENT_DIR` + `UMODEL_OUT`).
- `ue2-assets`: `properties.rs` atravessa bytecode antes dos defaults via `script.rs`, seguindo `UStruct::SerializeExpr` C4 (core.dll10132200), com contador lógico separado da posição no arquivo. Evidência e limites em `docs/evidence/class-script.md`; teste cobre todas as classes de Engine.u/LineageWarrior.u.
- `l2-catalog`: `Catalog::texture_raw` (com `RawTexture`/`RawMip`) expõe todos os mips de uma textura com os bytes crus (DXT ainda comprimido), para upload direto na GPU sem decodificar cada mip para RGBA.
- ue2-level: `bsp.rs` guarda a árvore do BSP (`BspNode`/`BspTree::zone_at`), lê `NumSharedSides`, `NumZones` e `Zones[]` (`ZoneActor` + `Connectivity` + `Visibility` + `LastRenderTime`) depois de `Verts`, e marca cada `BspPolygon` com a zona da frente; `lib.rs` expõe `Level.bsp_tree`, `Level.zones` e `Level.zone_infos` (novo `zones.rs`, código nosso).
- ue2-level: `ZoneProperties.terrain_zone` lê `bTerrainZone` do `ZoneInfo`, para a cena saber qual zona é a do terreno.
- `l2-crypto`: `decrypt_owned` decifra o arquivo no próprio buffer (sem segunda cópia) e `Window` decifra qualquer trecho do payload dos esquemas XOR (`Ver111`/`Ver121`, byte a byte) sem o resto do arquivo.
- `l2-catalog`: o `Catalog` guarda só o índice (`Package` + nomes dos exports) de cada pacote e lê do disco apenas os bytes do export pedido (`seek` + `read` + XOR no lugar) num buffer de trabalho reutilizado; nunca retém pacotes inteiros (antes guardava o arquivo todo de cada pacote aberto para sempre, mais de 500 MB só com os `.ukx` dos personagens). `Ver413` continua decifrado inteiro. `release_files()` solta o buffer; `cached_bytes()` mede o que está em memória.
- ue2-level: `Actor.sun_affect` lê `bSunAffect` (default `false`, como na classe `Actor`); `Light` (`location`, `radius` = 25·(LightRadius+1), `brightness`, `hue`, `saturation`, `light_type`, `effect`) lido por `pub fn light` de todo actor com `LightType` (classe `Light` default steady) e acumulado em `Level.lights`. Evidência em `docs/evidence/hardware-lights.md`.
- `ue2-assets/src/meshes/skeletal/lod.rs`: o osso de reserva das seções rígidas (`Bip01_head`) casa também `Bip01 head` — os `MOrc_m000..m003_m00_bh` do `Orc.ukx` nomeiam os ossos com espaço em vez de `_`.
- `ue2-assets/src/meshes/skeletal/{mod,lod}.rs`: `Layout::Unlicensed` para os `.ukx` customizados do L2Killer (versão 123, licensee 0): o layout 27 sem o array de f32 e o i32 finais do `SkeletalMesh`; nos LODs rígidos sem tabelas antigas, um esqueleto de um só osso prende a ele. `tests/l2killer_custom.rs` (ignorado; `KILLER_CLIENT_DIR`). Evidência em `docs/evidence/ukx-c4.md` § "Pacotes customizados".
- `l2-catalog`: `material.rs` reescrito como o material do `d3ddrv` (`Material { stages, passes, sorting_level }`, tabelas de `OutputBlending`/`FrameBufferBlending`/flags de `UTexture`, `Shader`/`Combiner`/`FinalBlend`/`ColorModifier`/`OpacityModifier`, specular multipass), `uv.rs` novo (`TexPanner`/`TexOscillator`/`TexRotator`/`TexScaler::GetMatrix` e `UTexture::Tick` como `frame_index`), `fade.rs` com o `UFadeColor::GetColor` do C4; exports homônimos de um pacote são distinguidos por `Group.Name#índice` nos caminhos que o catálogo devolve. Evidência em `docs/evidence/render-spec.md` §1 e `docs/renders/README.md`.
- `ue2-assets`: `Property.index` (índice de elemento dos arrays fixos, antes descartado) para ler `Cubemap.Faces[6]`; `StaticMesh.uvs2` com o segundo canal de UV (antes descartado) para `TexCoordSource`.
- ue2-level: `bsp.rs` lê o resto do `UModel` (`Polys`, `Bounds`, `LeafHulls`, `Leaves`, `Lights`, `Sections` com `FBspVertexStream`, `LightMaps`, `TimeLightMapTextures`/`LightMapTextures`) em `BspRender`, os campos `iSection/iFirstVertex/iLightMap` do nó em `NodeDraw`, a normal do plano em `BspPolygon.normal`; `0x2000` sai do `HIDDEN` (`render-spec.md` §2.7). `Level.bsp_render`.
- ue2-level: `lighting::sector` lê o `UTerrainSector` campo a campo (`render-spec.md` §3a.1.4: `LightInfos`, 8 intensity maps 17×17, `LayerAllCache`, `PassMask`) em vez da heurística; `terrain.rs` lê `EdgeTurnBitmap`, `Inverted`, pan/rotação/eixo das camadas e a cauda nativa do `ATerrainInfo` (`TerrainNative`: `ToWorld`, `ToHeightmap`, `HeightmapX/Y`, `VertexColors`).
- `ue2-assets/src/meshes/static_mesh.rs`: `Section` guarda `primitives` (`NumPrimitives`, o que o engine desenha) e `strip` em vez de `NumTriangles` (só estatística); o leitor segue depois do `IndexBuffer` (`WireframeIndexBuffer`, `CollisionModel`, os dois `TLazyArray` saltados pelo `SeekPos` com `Reader::seek`) e lê a cauda L2 por `LicenseeVer` (`StaticMesh.lod: MeshLod { mesh01, mesh02, range01, range02, blend }`, `sway: Sway { frequency, max_angle }`, `billboard`). `l2-catalog::Mesh.lod_meshes` resolve os caminhos das duas LODs. Evidência em `docs/evidence/render-spec.md` §6.1/§6.6.
- ue2-level: `Actor.hidden/is_static/ignored_range/time_reactor` (`bHidden`, `bStatic`, `bIgnoredRange`, `bTimeReactor` + `ShowTime/HideTime`); `Light.emitter_light_type/time_light`; `DecoLayer` completo (`AlignToTerrain`, `ScaleMap`, `DrawOrder`, `LitDirectional`, `bForceRender`) e `Terrain.deco_layer_offset`; `sky.rs` lê `SkyZoneInfo` (`LensFlare[12]`, offsets, escalas), `NSun`/`NMoon` (`Radius`, `Skins[0]`, `SunScale`/`MoonScale`, `EnvType`) em `Level.sky`.
- ue2-level: `fog.rs` lê `L2FogInfo` (`AffectRange`, `FogRange1..5`, `TextureDistance`, `Colors[]` com `CloudColor[]`/`HazeRingColor[]`, `CloudTexture`) em `Level.fog_infos`; `*FogInfo` deixa de ser lido como zona; `Fog.blend_time` (`DistanceFogBlendTime`) e `ZoneProperties.clear_to_fog_color`; `ue2-assets::Property::colors` (array de `Color`).

- `ue2-assets`: `sound::read_sound` lê FileType e TLazyArray de USound conforme Engine
  `10503b50/10377710/10375e30`; `l2-catalog::Catalog::sound` resolve um export Sound pelo
  caminho. Teste sobre todos os341 sons de ItemSound/InterfaceSound. Evidência em
  `docs/evidence/ui-audio-app.md`, seção Som da interface e USound serializado.

- `ue2-core`: ByteWindow conserva offsets absolutos sem armazenar o prefixo;
  leitores de exports em `ue2-assets` aceitam a janela. `ue2-package` usa um
  parser comum para slice e Read+Seek; `l2-catalog` indexa sem alocar pacote
  inteiro nos formatos Plain/111/121 e lê só serial_size. `l2-crypto` confere
  tamanho/CRC do payload por streaming. Ver413 liberado é recarregado quando
  necessário. Evidência: `docs/evidence/loading-memory.md`.

- `l2-catalog`: o acesso a arquivos passa por `killer-platform::fs`. No nativo
  é o próprio `std::fs`; no Worker WASM usa arquivos selecionados pelo usuário,
  com `Read + Seek` via Blob/FileReaderSync. Não altera decifragem nem formatos.
  Evidência: `docs/evidence/web-platform.md`.
