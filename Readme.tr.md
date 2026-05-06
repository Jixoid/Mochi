## Mochi

C++ ile yazılmış Vulkan tabanlı, minimalistik bir grafik motoru ve framework'ü.
QAOS, düşük seviyeli grafik programlamayı basitleştirmek için temiz, katmanlı bir mimariye odaklanmaktadır.

Diğer diller: [en](Readme.md)


## Mimari

Proje 3 temel katmana ayrılmıştır:

1. **RHI (Render Donanım Arayüzü):** Düşük seviyeli GPU kaynak yönetimi.
2. **Dünya:** Düğümler, Kameralar, Işıklar ve Görseller dahil olmak üzere sahne nesneleri.
3. **Varlık:** Mesh'ler gibi yüksek seviyeli veri yapıları.


## Görseller

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

Bu proje GNU Genel Kamu Lisansı versiyon 3 (GPL3) kapsamında lisanslanmıştır. Detaylar için LICENSE klasörüne bakın.

Copyright (c) 2025-2026 Kadir Aydın.


## QAOS

Açık kaynak topluluğu tarafından ❤️ ile geliştirildi.
