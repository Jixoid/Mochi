## Mochi

C++ ile yazılmış Vulkan tabanlı, minimalistik bir grafik motoru ve framework'ü.
QAOS, düşük seviyeli grafik programlamayı basitleştirmek için temiz, katmanlı bir mimariye odaklanmaktadır.

Diğer diller: [en](README.md)


## Mimari

Proje 3 temel katmana ayrılmıştır:

1. **RHI (Render Donanım Arayüzü):** Düşük seviyeli GPU kaynak yönetimi.
2. **Dünya:** Düğümler, Kameralar, Işıklar ve Görseller dahil olmak üzere sahne nesneleri.
3. **Varlık:** Mesh'ler gibi yüksek seviyeli veri yapıları.


## Görseller

![Su Şişesi 3D Model](/images/WaterBottle.png)
*PBR (Fizik Tabanlı Render) render testi.*

![Avakado 3D Modeli](/images/Avacado.png)
*PBR (Fizik Tabanlı Render) render testi.*

![Suzanne 3D Modeli](/images/Suzanne.png)
*PBR (Fizik Tabanlı Render) render testi.*


## Derleme

Bu proje derleme sistemi olarak CMake kullanıyor.
Basitçe aşşağıdaki şekilde de derleyebilirsiniz.

```bash
scripts/configure.sh
scripts/build.sh
```


## Lisans

Bu proje GNU Genel Kamu Lisansı versiyon 3 (GPL3) kapsamında lisanslanmıştır.

Copyright (c) 2025-2026 Kadir Aydın.


## QAOS

Açık kaynak topluluğu tarafından ❤️ ile geliştirildi.
