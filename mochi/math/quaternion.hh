/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <type_traits>
#include <cmath>
#include "mochi/math/vec3.hh"



namespace mochi
{

    /** 
   * @brief A generic quaternion for representing rotations.
   * @tparam T The element type.
   */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct quaternion
  {
    public:
      T W{1}, X{}, Y{}, Z{};

    public:
      /** @brief Default constructor (Identity quaternion). */
      explicit inline quaternion() {}
      /** @brief Construct with specific W, X, Y, and Z values. */
      inline quaternion(T nW, T nX, T nY, T nZ): W(nW), X(nX), Y(nY), Z(nZ) {}

      /**
       * @brief Create a quaternion from an axis and an angle.
       * @param angle The rotation angle in radians.
       * @param axis The rotation axis.
       * @return A quaternion representing the rotation.
       */
      [[nodiscard]] inline static fun fromAxisAngle(T angle, const vec3<T> &axis) -> quaternion {
        axis = axis.normalize();
        T halfAngle = angle *0.5f;
        T s = std::sin(halfAngle);

        return quaternion(
          std::cos(halfAngle),
          axis.X *s,
          axis.Y *s,
          axis.Z *s
        );
      }

      /**
       * @brief Create a quaternion that rotates vector u to vector v.
       * @param u The source vector.
       * @param v The destination vector.
       * @return A quaternion representing the rotation from u to v.
       */
      [[nodiscard]] inline static fun fromTwoVector(vec3<T> u, vec3<T> v) -> quaternion  {
        u = u.normalize();
        v = v.normalize();
        
        T cosTheta = u.dot(v);
        vec3<T> rotationAxis;

        if (cosTheta < -1.0 + 1e-6) {
          rotationAxis = vec3<T>{0, 1, 0}.cross(u);
          if (rotationAxis.dist() < 0.01)
            rotationAxis = vec3<T>{1, 0, 0}.cross(u);
          return fromAxisAngle(M_PI, rotationAxis.normalize());
        }

        rotationAxis = u.cross(v);
        T s = std::sqrt((1.0 + cosTheta) * 2.0);
        T invS = 1.0 / s;

        return quaternion(
          s * 0.5,
          rotationAxis.X * invS,
          rotationAxis.Y * invS,
          rotationAxis.Z * invS
        ).normalize();
      }

      /**
       * @brief Create a look-at rotation quaternion.
       * @param pos The eye position.
       * @param target The target position to look at.
       * @param up The up vector.
       * @return A quaternion representing the look-at rotation.
       */
      [[nodiscard]] inline static fun lookAt(const vec3<T> &pos, const vec3<T> &target, const vec3<T> &up) -> quaternion  {
        vec3<T> forward = (target - pos).normalize();
        
        vec3<T> right = forward.cross(up).normalize();
        vec3<T> orthoUp = right.cross(forward).normalize();
        
        quaternion q1 = fromTwoVector(vec3<T>(0, 0, -1), forward);
        
        vec3<T> currentUp = q1.rotate(vec3<T>(0, 1, 0));
        
        quaternion q2 = fromTwoVector(currentUp, orthoUp);
        
        return q2 * q1;
      }

      /**
       * @brief Spherical linear interpolation between two quaternions.
       * @param q1 The starting quaternion.
       * @param q2 The ending quaternion.
       * @param t The interpolation parameter [0, 1].
       * @return The interpolated quaternion.
       */
      [[nodiscard]] inline static fun slerp(quaternion q1, quaternion q2, T t) -> quaternion  {
        q1 = q1.normalize();
        q2 = q2.normalize();

        T dot = (q1.W * q2.W) + (q1.X * q2.X) + (q1.Y * q2.Y) + (q1.Z * q2.Z);

        if (dot < 0.0f) {
          q2 = quaternion(-q2.W, -q2.X, -q2.Y, -q2.Z);
          dot = -dot;
        }

        if (dot > 0.9995)
          return quaternion(
            q1.W + t * (q2.W - q1.W),
            q1.X + t * (q2.X - q1.X),
            q1.Y + t * (q2.Y - q1.Y),
            q1.Z + t * (q2.Z - q1.Z)
          ).normalize();

        T theta_0 = std::acos(dot);
        T theta = theta_0 * t;
        T sin_theta_0 = std::sin(theta_0);
        T sin_theta = std::sin(theta);

        T s0 = std::cos(theta) - dot * sin_theta / sin_theta_0;
        T s1 = sin_theta / sin_theta_0;

        return quaternion(
          (s0 * q1.W) + (s1 * q2.W),
          (s0 * q1.X) + (s1 * q2.X),
          (s0 * q1.Y) + (s1 * q2.Y),
          (s0 * q1.Z) + (s1 * q2.Z)
        );
      }


    public:
      inline fun operator*(const quaternion &it) const -> quaternion {
        return quaternion(
          W *it.W - X *it.X - Y *it.Y - Z *it.Z,  // w
          W *it.X + X *it.W + Y *it.Z - Z *it.Y,  // x
          W *it.Y - X *it.Z + Y *it.W + Z *it.X,  // y
          W *it.Z + X *it.Y - Y *it.X + Z *it.W   // z
        );
      }

      
    public:
      /**
       * @brief Rotate a 3D vector by this quaternion.
       * @param v The vector to rotate.
       * @return The rotated vector.
       */
      [[nodiscard]] inline fun rotate(const vec3<T> &v) const -> vec3<T> {
        vec3<T> u{X,Y,Z};

        vec3<T> t = u.cross(v) *2; 
        
        return v + (t *W) +u.cross(t);
      }

      /** @brief Get the conjugate of this quaternion. */
      [[nodiscard]] inline fun conjugate() const -> quaternion {
        return quaternion(W, -X, -Y, -Z);
      }

      /** @brief Return a normalized copy of this quaternion. */
      [[nodiscard]] inline fun normalize() const -> quaternion {
        T len = std::sqrt(W*W + X*X + Y*Y + Z*Z);
        if (len > 0) {
          T invLen = 1.0 / len;

          return quaternion(W*invLen, X*invLen, Y*invLen, Z*invLen);
        }
        return quaternion();
      }
    
  };

}
