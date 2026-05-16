## Mochi

Vulkan 1.4 tabanlı modern bir render motoru ve altyapısı (framework). 
QAOS çatısı altında geliştirilen Mochi, modern prensipleri benimserken düşük seviyeli grafik programlamayı basitleştirmek için temiz ve katmanlı bir mimariye odaklanır.

Diğer diller: [en](README.md)


## Özellikler

Mochi, minimalist bir sarmalayıcı olmanın ötesine geçerek yüksek performanslı render işlemleri için sağlam bir temel sunar:

* **Modern RHI:** `vulkan-hpp` (RAII) kullanılarak Vulkan 1.4 üzerinde inşa edilmiştir. Dinamik Render (Dynamic Rendering - eski nesil RenderPass/Framebuffer kalabalığı yok) ve modern veri bağlama için Buffer Device Address (BDA) özelliklerini içerir.
* **Varlık Bileşen Sistemi (ECS):** Önbellek dostu (cache-friendly), esnek sahne ve nesne yönetimi (Transform'lar, Kameralar, Işıklar, Mesh'ler) için `EnTT` altyapısını kullanır.
* **Akıllı Bellek ve Kaynak Yönetimi:** Vulkan Memory Allocator (VMA) ile entegredir. Darboğazsız kaynak yüklemesi için otomatik ve toplu (batched) bir transfer kuyruğu sistemine sahiptir.
* **Sanal Dosya Sistemi (VFS):** Varlıkları (assets) hem fiziksel disk depolamasından hem de derlenmiş gömülü (embedded) dosyalardan sorunsuzca çözen özel bir VFS.
* **SIMD Hızlandırmalı Matematik:** Ağır hesaplama yükleri için SSE/AVX komutlarıyla (intrinsics) optimize edilmiş özel bir matematik kütüphanesi.
* **Varlık Akışı (Asset Pipeline):** Kullanıma hazır PBR materyal iş akışının yanı sıra yerleşik `.gltf` / `.glb` ve `.obj` ayrıştırma (parsing) desteği.


## Mimari

Proje, birbirinden olabildiğince bağımsız (highly decoupled) modüller halinde yapılandırılmıştır:

1. **RHI (Render Donanım Arayüzü):** Düşük seviyeli GPU kaynak yönetimi, pipeline'lar ve descriptor'lar.
2. **ECS (Dünya):** Veri odaklı sahne varlıkları, transform'lar ve ışıklandırma temsili.
3. **Asset & VFS:** Yüksek seviyeli veri yapıları (Mesh'ler, Dokular) ve donanımdan bağımsız dosya yükleme.
4. **Çekirdek (Core) Modüller:** Ekran, Bellek, Cihaz ve Render döngülerinin birleşik yönetimi.


## Görseller

![WaterBottle 3D Model](/images/WaterBottle.png)
*PBR render testi.*

![Avacado 3D Model](/images/Avacado.png)
*PBR render testi.*

![Suzanne 3D Model](/images/Suzanne.png)
*PBR render testi.*


## Lisans

Bu proje GNU Genel Kamu Lisansı sürüm 3 (GPL3) altında lisanslanmıştır.

Telif Hakkı (c) 2025-2026 Kadir Aydın.


## QAOS

Açık kaynak topluluğu tarafından ❤️ ile geliştirildi.
