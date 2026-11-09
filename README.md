# Laboratorio: Shaders Procedurales - Sistema Solar

Proyecto desarrollado en Rust mediante un renderizador 3D por software propio para el curso de Gráficas por Computadora.

El objetivo del laboratorio es diseñar y renderizar múltiples cuerpos celestes utilizando exclusivamente **shaders de fragmentos procedurales en CPU** (sin texturas ni materiales precargados).

## Arquitectura y Pipeline Base
- **Álgebra Lineal y Transformaciones:** Implementación propia de `Vec2`, `Vec3`, `Vec4`, `Mat4`, proyecciones en perspectiva y Look-At.
- **Buffers de CPU:** `Framebuffer` de color RGBA y `DepthBuffer` (Z-Buffer) administrados en memoria principal.
- **Rasterizador por Software:** Rasterización paralela por franjas horizontales (`std::thread::scope`), recorte de primitivas contra el plano cercano (*clipping*) e interpolación con corrección de perspectiva.
- **Cámara Orbital Panorámica:** Control esférico suave (`yaw`, `pitch`, `distance`) operable con las flechas del teclado, adaptado para encuadrar el sistema planetario.
- **Cinemática Orbital:** Rotación sobre eje propio y traslación orbital analítica continuas en función del tiempo transcurrido (`sim_time`).

## Estrella Solar: Shader de Plasma Procedural
La estrella central del sistema solar se evalúa completamente en CPU a través de 4 capas analíticas que modulan el color y brillo superficial en función de coordenadas continuas y tiempo:

1. **Capa 1: Cromosfera y Gradiente Térmico:** Mapea la emisión de cuerpo negro desde rojos carmesí profundos en las capas frías `(0.7, 0.08, 0.0)` hasta amarillos y blancos de alta energía térmica `(1.0, 0.95, 0.85)`.
2. **Capa 2: Macro-ondas de Convección (Llamaradas):** Tres funciones sinusoidales continuas acopladas tridimensionalmente que se desplazan armónicamente con la velocidad de convección solar.
3. **Capa 3: Micro-granulación Fotosférica:** Estructura de alta frecuencia que simula las células de ebullición del plasma en la superficie.
4. **Capa 4: Manchas Solares y Resplandor de Corona:**
   - **Manchas solares:** Vórtices oscuros calculados trigonométricamente simulando zonas magnéticas donde se inhibe la transferencia térmica.
   - **Corona / Efecto Limb:** Resplandor de silueta basado en Fresnel `(1 - N·V)^2.8` que irradia luz dorada hacia el vacío del espacio exterior.

## Planeta Rocoso: Shader Geológico Procedural
Superficie planetaria rocosa generada matemáticamente mediante síntesis analítica y ruido determinista 3D:

1. **Capa 1: Cuencas Basálticas y Mares Volcánicos:** Regiones bajas de baja frecuencia modeladas con ruido de valor continuo cúbico en tonos oscuros `(0.12, 0.10, 0.09)`.
2. **Capa 2: Mesetas y Masas Continentales:** Terreno elevado intermedio con tonalidades ocres, arcillosas y terracota mineral `(0.28, 0.20, 0.17)`.
3. **Capa 3: Cráteres de Impacto con Crestas Elevadas:** Patrones circulares generados mediante *ridged noise* `(1 - |ruido - 0.5| * 2)^2` con bordes claros y depósitos de eyecta.
4. **Capa 4: Micro-regolito e Iluminación Fotométrica Solar:** Grano granular de alta frecuencia superficial acoplado a un modelo de iluminación difusa originado en el Sol central `(0, 0, 0)` con modulación de relieve.
