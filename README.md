# Laboratorio: Shaders Procedurales - Sistema Solar

Proyecto desarrollado en Rust mediante un renderizador 3D por software propio para el curso de Gráficas por Computadora.

El objetivo del laboratorio es diseñar y renderizar múltiples cuerpos celestes utilizando exclusivamente **shaders de fragmentos procedurales en CPU** (sin texturas ni materiales precargados).

## Cuerpos celestes contemplados
- **Estrella (Sol):** Shader de plasma solar multicapa con convección, células térmicas y manchas solares.
- **Planeta Rocoso:** Shader geológico procedural con cráteres de impacto y mares de basalto.
- **Planeta Tierra:** Superficie oceánica con masas continentales, casquetes polares y capa de nubes/atmósfera.
- **Gigante Gaseoso:** Bandas atmosféricas jovianas, gradientes de latitud y turbulencia de vientos.
## Fase 2: Pipeline Base de Renderizado y Esfera Central
- Módulos matemáticos completos (`Vec2`, `Vec3`, `Vec4`, `Mat4`).
- Pipeline de software en CPU con rasterización paralela por franjas, buffer de profundidad (Z-Buffer) y clipping contra el plano cercano.
- Carga de mallas Wavefront OBJ (`sphere.obj`) abstrayendo coordenadas geométricas puras (sin materiales ni texturas).
- Cámara orbital interactiva controlada con las flechas del teclado.
- Renderizado de la esfera central en el origen `(0, 0, 0)` con rotación axial continua en función del tiempo.

